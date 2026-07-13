use iced::widget::container::background;
use iced::widget::pane_grid::Pane;
use iced::widget::{button, column, pane_grid, row, text, text_editor, Container, Row};
use iced::{Element, Padding, Theme};

fn main() -> iced::Result {
    iced::run(MdpGraph::update, MdpGraph::view)
}

struct MdpGraph {
    pane_grid: pane_grid::State<TabView>,
}

struct TabView {
    tabs: Vec<Window>,
    selected: usize,
}

impl TabView {
    fn new() -> Self {
        Self {
            tabs: Vec::new(),
            selected: 0,
        }
    }
}

enum Window {
    TextEditor(text_editor::Content),
    GraphView(i64),
}

impl Default for MdpGraph {
    fn default() -> Self {
        MdpGraph::new()
    }
}

impl MdpGraph {
    fn new() -> Self {
        let (pane_grid, _) = pane_grid::State::new(TabView::new());
        Self { pane_grid }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::EditText {
                pane,
                tab_index,
                action,
            } => match &mut self.pane_grid.get_mut(pane).unwrap().tabs[tab_index] {
                Window::TextEditor(state) => state.perform(action),
                Window::GraphView(_) => {
                    panic!("Window type mismatch!")
                }
            },
            Message::CounterAction {
                pane,
                tab_index,
                action,
            } => match &mut self.pane_grid.get_mut(pane).unwrap().tabs[tab_index] {
                Window::TextEditor(_) => {
                    panic!("Window type mismatch!")
                }
                Window::GraphView(state) => match action {
                    CounterAction::Increment => *state += 1,
                    CounterAction::Decrement => *state -= 1,
                },
            },

            Message::OpenText { pane } => {
                let pane = self.pane_grid.get_mut(pane).unwrap();
                pane.tabs
                    .push(Window::TextEditor(text_editor::Content::new()));
                pane.selected = pane.tabs.len() - 1
            }
            Message::OpenGraph { pane } => {
                let pane = self.pane_grid.get_mut(pane).unwrap();
                pane.tabs.push(Window::GraphView(0));
                pane.selected = pane.tabs.len() - 1
            }
            Message::SelectTab { pane, tab_index } => {
                self.pane_grid.get_mut(pane).unwrap().selected = tab_index
            }
            Message::Split { pane, axis } => {
                self.pane_grid.split(axis, pane, TabView::new());
            }
            Message::PaneResized(action) => self.pane_grid.resize(action.split, action.ratio),
        }
    }

    fn view<'a>(&'a self) -> Element<'a, Message> {
        column![self.hotbar(), row![self.project_bar(), self.main_window()]].into()
    }

    fn hotbar<'a>(&'a self) -> Element<'a, Message> {
        Container::new(
            row![button("+"), button("-"), button("*"), button("/")].padding(Padding::new(10.0)),
        )
        .style(|a: &Theme| background(a.palette().background))
        .into()
    }

    fn project_bar<'a>(&'a self) -> Element<'a, Message> {
        column![
            button("project 1"),
            button("project 2"),
            button("project 3"),
            button("+")
        ]
        .into()
    }

    fn main_window<'a>(&'a self) -> Element<'a, Message> {
        pane_grid::PaneGrid::new(&self.pane_grid, |id, state, maximised| {
            pane_grid::Content::new(self.tabbed_window(id, state))
        })
        .on_resize(2, |resize| Message::PaneResized(resize))
        .into()
    }

    fn tabbed_window<'a>(&'a self, pane: Pane, state: &'a TabView) -> Element<'a, Message> {
        let mut tab_bar = Row::new();
        for (tab_index, tab) in state.tabs.iter().enumerate() {
            tab_bar = tab_bar.push(button("tab").on_press(Message::SelectTab { pane, tab_index }));
        }
        tab_bar = tab_bar.push(button("+").on_press(Message::OpenText { pane }));
        tab_bar = tab_bar.push(button("O").on_press(Message::OpenGraph { pane }));
        tab_bar = tab_bar.push(button("|").on_press(Message::Split {
            pane,
            axis: pane_grid::Axis::Vertical,
        }));
        tab_bar = tab_bar.push(button("--").on_press(Message::Split {
            pane,
            axis: pane_grid::Axis::Horizontal,
        }));

        let main_window = if state.tabs.len() == 0 {
            text!("This tab view is empty").into()
        } else {
            let selected = &state.tabs[state.selected];
            match selected {
                Window::TextEditor(window_state) => {
                    self.code_window(pane, state.selected, window_state)
                }
                Window::GraphView(window_state) => {
                    self.graph_view(pane, state.selected, *window_state)
                }
            }
        };

        column![tab_bar, main_window].into()
    }

    fn code_window<'a>(
        &'a self,
        pane: Pane,
        tab_index: usize,
        content: &'a text_editor::Content,
    ) -> Element<'a, Message> {
        text_editor(content)
            .placeholder("Document is empty")
            .on_action(move |action| Message::EditText {
                pane,
                tab_index,
                action,
            })
            .into()
    }

    fn graph_view<'a>(
        &'a self,
        pane: Pane,
        tab_index: usize,
        content: i64,
    ) -> Element<'a, Message> {
        column![
            button("+").on_press(Message::CounterAction {
                pane,
                tab_index,
                action: CounterAction::Increment
            }),
            text(content),
            button("-").on_press(Message::CounterAction {
                pane,
                tab_index,
                action: CounterAction::Decrement
            }),
        ]
        .into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    EditText {
        pane: pane_grid::Pane,
        tab_index: usize,
        action: text_editor::Action,
    },
    CounterAction {
        pane: pane_grid::Pane,
        tab_index: usize,
        action: CounterAction,
    },
    OpenText {
        pane: Pane,
    },
    OpenGraph {
        pane: Pane,
    },
    SelectTab {
        pane: Pane,
        tab_index: usize,
    },
    Split {
        pane: Pane,
        axis: pane_grid::Axis,
    },
    PaneResized(pane_grid::ResizeEvent),
}

#[derive(Debug, Clone)]
enum CounterAction {
    Increment,
    Decrement,
}
