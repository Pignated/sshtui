use std::{cell::RefCell, sync::Arc};

use markdown_ast::{Block, markdown_to_ast};
use ssh_ui::{
    App, AppSession, cursive::{
        Cursive, Printer, View, theme::{Effect, Palette, Style, Theme}, utils::{markup::StyledString, span::SpannedString}, view::{Finder, Nameable, Resizable, SizeConstraint}, views::{
            Button, Dialog, EditView, LinearLayout, NamedView, ResizedView, ScrollView, TextView
        }
    }
};
use tokio::{runtime::Handle, sync::broadcast};

pub struct TestApp {
    broadcast_tx: broadcast::Sender<SpannedString<Style>>,
    user_tx: broadcast::Sender<SpannedString<Style>>,
    palette: Palette
}

impl TestApp {
    pub fn new(broadcast_tx: broadcast::Sender<SpannedString<Style>>, user_tx: broadcast::Sender<SpannedString<Style>>, palette: Palette) -> Self {
        Self {
            broadcast_tx,
            user_tx,
            palette
        }
    }
}
impl App for TestApp {
    fn on_load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
 
    }

    fn new_session(&self) -> Box<dyn ssh_ui::AppSession> {
        let sender = self.user_tx.clone();
        let receiver = self.broadcast_tx.subscribe();
        Box::new(TestAppSession::new(sender,receiver, self.palette.clone()))
    }
}

struct TestAppSession {
    sender: broadcast::Sender<SpannedString<Style>>,
    receiver: broadcast::Receiver<SpannedString<Style>>,
    palette: Palette,
}

impl TestAppSession {
    pub fn new(sender: broadcast::Sender<SpannedString<Style>>, receiver: broadcast::Receiver<SpannedString<Style>>, palette: Palette) -> Self {
        Self {sender, receiver, palette}
    }
}

impl AppSession for TestAppSession {
    fn on_start(
        &mut self,
        siv: &mut ssh_ui::cursive::Cursive,
        _session_handle: ssh_ui::SessionHandle,
        pub_key: Option<ssh_ui::russh_keys::key::PublicKey>, // Can be used as identity
        _force_refresh_sender: std::sync::mpsc::Sender<()>,
    ) -> Result<Box<dyn ssh_ui::cursive::View>, Box<dyn std::error::Error>> {
        siv.set_autorefresh(true);
        let mut outer_window = LinearLayout::vertical();
        let inside_scroll = LinearLayout::vertical().with_name("scrollWindow");
        let scroll_window = ScrollView::new(inside_scroll).with_name("outerScroll").min_height(15).with_name("outerScrollSize");
        let mut lower_window = LinearLayout::horizontal();
        let username = match pub_key {
            Some(key) => {
                let key_copy = key.clone();
                let finger = key_copy.fingerprint().clone();
                match finger.get(0..5) {
                    Some(str) => str.to_string(),
                    None => "Anon".to_string(),
                }
            }
            None => "Anon".to_string(),
        };
        let shared_user = Arc::new(username.to_owned());
        let local_user = shared_user.clone();
        let local_sender = self.sender.clone();
        let text_view = EditView::new()
            .on_submit_mut(move |s, txt| {
                s.call_on_name("editBox", |view:&mut EditView| {
                    view.set_content("");
                });
                send_message(s, txt.to_string(), local_user.clone(),&local_sender);
            })
            .with_name("editBox")
            .max_height(1)
            .with_name("editBoxSize")
            .min_width(50);
        let local_sender = self.sender.clone();
        let local_user_2 = shared_user.clone();
        let send_button = Button::new("Send", move |s| {
            let txt = s.call_on_name("editBox", |view: &mut EditView| {
                let txt = view.get_content();
                view.set_content("");
                (*txt).clone()
            });
            let txt  = match txt {
                Some(txt) => txt,
                None => return
            };
            send_message(s, txt, local_user_2.clone(), &local_sender);
        });
        let quit_button = Button::new("Quit", |s| {
            s.quit();
        });
        lower_window.add_child(text_view);
        lower_window.add_child(send_button);
        lower_window.add_child(quit_button);
        let lower_window = lower_window.full_width();
        outer_window.add_child(scroll_window);
        outer_window.add_child(lower_window);
        let theme = Theme {
            shadow: false,
            palette: self.palette.clone(),
            ..Default::default()
        };
        let outer_window = ListenHandler{
            inner: RefCell::new(outer_window),
            rx: RefCell::new(self.receiver.resubscribe())
        };
        let _ = siv.cb_sink().send(Box::new(|s| {
            s.call_on_name("editBoxSize", |view: &mut ResizedView<NamedView<EditView>>| {
                view.set_constraints(SizeConstraint::Full , SizeConstraint::Fixed(1));
            });
            s.call_on_name("outerScrollSize", |view: &mut ResizedView<NamedView<ScrollView<NamedView<LinearLayout>>>>| {
                view.set_constraints(SizeConstraint::Full, SizeConstraint::Full);
            });
            s.set_window_title("SSH Chat");//TODO get from config

        }));
        siv.set_theme(theme);
        let outer_window = Dialog::around(outer_window).title("SSH Chat"); // TODO get from config
        Ok(Box::new(outer_window))
    }
}

struct ListenHandler {
    inner: RefCell<LinearLayout>,
    rx: RefCell<broadcast::Receiver<SpannedString<Style>>>
}

impl View for ListenHandler {
    fn draw(&self, printer: &Printer) {
        let mut inner = self.inner.borrow_mut();
        let is_on_bottom = inner.call_on_name("outerScroll",|view: &mut ScrollView<NamedView<LinearLayout>>| {
            view.is_at_bottom()
        }).unwrap_or_default();
        while let Ok(str) = self.rx.borrow_mut().try_recv() {
            inner.call_on_name("scrollWindow", |view: &mut LinearLayout| {
                let new_chat = TextView::new(str);
                view.add_child(new_chat);
                if view.len() > 1000 {
                    while view.len() > 1000 {
                        view.remove_child(0);
                    }
                }
            });
            if is_on_bottom {
                inner.call_on_name("outerScroll", |view: &mut ScrollView<NamedView<LinearLayout>>| {
                    view.scroll_to_bottom();
                });

            }
        }
        inner.draw(printer);
    }
    fn layout(&mut self, size: ssh_ui::cursive::Vec2) {
       self.inner.get_mut().layout(size);
    }
    fn required_size(&mut self, constraint: ssh_ui::cursive::Vec2) -> ssh_ui::cursive::Vec2 {
        self.inner.get_mut().required_size(constraint)
    }
    fn on_event(&mut self, event: ssh_ui::cursive::event::Event) -> ssh_ui::cursive::event::EventResult {
       self.inner.get_mut().on_event(event)
    }
    fn focus_view(
            &mut self,
            sel: &ssh_ui::cursive::view::Selector,
        ) -> Result<ssh_ui::cursive::event::EventResult, ssh_ui::cursive::view::ViewNotFound> {
       self.inner.get_mut().focus_view(sel)
    }
    fn type_name(&self) -> &'static str {
        "ListenHandler"
    }
    fn take_focus(
            &mut self,
            source: ssh_ui::cursive::direction::Direction,
        ) -> Result<ssh_ui::cursive::event::EventResult, ssh_ui::cursive::view::CannotFocus> {
        self.inner.get_mut().take_focus(source)
    }
    fn call_on_any(&mut self, sel: &ssh_ui::cursive::view::Selector, cb: ssh_ui::cursive::event::AnyCb) {
       self.inner.get_mut().call_on_any(sel, cb);
    }
    fn needs_relayout(&self) -> bool {
        self.inner.borrow().needs_relayout()
    }
    fn important_area(&self, view_size: ssh_ui::cursive::Vec2) -> ssh_ui::cursive::Rect {
        self.inner.borrow().important_area(view_size)
    }
}


fn create_message(text: String, username: Arc<String>) -> SpannedString<Style> {
    //TODO Use markdown_ast to convert text to Inline, then work through that tree to generate
    //the different ways to effect text. Likely this will require a separate parsing function that
    //returns SpannedString and is recursive
    let mut message_text = StyledString::styled((*username).clone(),Effect::Bold);
    message_text.append_plain(": ");
    message_text.append_plain(text);
    message_text
}
fn parse_message_text(text:String) -> SpannedString<Style> {
    parse_inline(markdown_to_ast(&text))
}
fn parse_inline(text: Vec<Block>) ->SpannedString<Style> {
    todo!()
}
fn send_message(s: &mut Cursive, txt: String, user: Arc<String>, sender: &broadcast::Sender<SpannedString<Style>>) {
    if txt.chars().all(char::is_whitespace) {
        s.call_on_name("editBox", |view: &mut EditView | {
            view.set_content("");
        });
        return
    }
    let out_str = create_message(txt, user);
    let sender_copy = sender.clone();
    let handle = Handle::current();
    handle.block_on( async {
        let _ = sender_copy.send(out_str);
    });
}   
