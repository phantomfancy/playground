#![windows_subsystem = "windows"]

use windows_reactor::*;

#[derive(Clone)]
enum Message {
    Increment,
    Reset,
}

struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Self {
        Self { count: 0 }
    }

    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Increment => self.count = self.count.saturating_add(1),
            Message::Reset => self.count = 0,
        }
    }
}

impl Component for Counter {
    type Input = ();
    type Message = Message;

    fn create(_input: &Self::Input, _context: &ComponentContext<Self>) -> Self {
        Self::new()
    }

    fn update(&mut self, message: Self::Message, _context: &ComponentContext<Self>) {
        self.handle_message(message);
    }

    fn view(&self, _input: &Self::Input, context: &mut ViewContext<Self>) -> View {
        context.window_title("Rust WinUI 计数器");

        StackPanel::new()
            .spacing(16.0)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center)
            .children((
                TextBlock::new()
                    .text("Rust WinUI 计数器")
                    .font_size(28.0)
                    .horizontal_alignment(HorizontalAlignment::Center),
                TextBlock::new()
                    .text(format!("当前计数：{}", self.count))
                    .font_size(20.0)
                    .horizontal_alignment(HorizontalAlignment::Center),
                StackPanel::new()
                    .orientation(Orientation::Horizontal)
                    .spacing(8.0)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .children((
                        Button::new()
                            .on_click(context.message(Message::Increment))
                            .content("增加"),
                        Button::new()
                            .on_click(context.message(Message::Reset))
                            .content("重置"),
                    )),
            ))
    }
}

fn main() {
    App::run_component::<Counter>(()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increments_and_resets_the_counter() {
        let mut counter = Counter::new();

        counter.handle_message(Message::Increment);
        counter.handle_message(Message::Increment);
        assert_eq!(counter.count, 2);

        counter.handle_message(Message::Reset);
        assert_eq!(counter.count, 0);
    }
}
