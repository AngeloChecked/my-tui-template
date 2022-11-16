use std::{collections::HashMap, rc::Rc};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::DOT,
    text::Spans,
    widgets::{Block, Borders, Tabs},
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

use std::cell::RefCell;

enum Selection<'b> {
    TextArea(TextArea<'b>),
    Tab(usize, Rc<RefCell<Tabs<'b>>>),
}

enum SelectionMove {
    Right,
    Left,
    Up,
    Down,
}

impl<'s> Selection<'s> {
    fn textarea(&mut self) -> &mut TextArea<'s> {
        if let Selection::TextArea(textarea) = self {
            return textarea;
        } else {
            unreachable!()
        }
    }
}

pub struct App<'a> {
    tag_matrix: Vec<Vec<&'a str>>,
    tag_map: HashMap<&'a str, Selection<'a>>,
    current_coo: (usize, usize),
    last_tab_coo: (usize, usize),
}

impl<'app> App<'app> {
    pub fn new() -> App<'app> {
        let mut map = HashMap::new();
        let tab_number = 0;
        let titles: Vec<Spans> = ["Tab1", "Tab2"].iter().cloned().map(Spans::from).collect();

        let tabs = Rc::new(RefCell::new(
            Tabs::<'app>::new(titles.clone())
                .block(Block::default().title("Tabs").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .select(tab_number)
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(DOT),
        ));

        let tabs_clone: Rc<RefCell<Tabs>> = Rc::clone(&tabs);
        map.insert("tab1", Selection::<'app>::Tab(0, tabs_clone));
        map.insert("tab2", Selection::<'app>::Tab(1, tabs));

        let mut textarea1 = TextArea::<'app>::default();
        App::_textarea_selected(&mut textarea1);
        let text_raw = "sdfjasdf\nskadjfskdjf\nadfjklsdf\nasddsf";
        for text in text_raw.split('\n') {
            textarea1.insert_str(text);
            textarea1.insert_newline();
        }

        let mut textarea2 = TextArea::<'app>::default();
        App::_textarea_unselected(&mut textarea2);

        map.insert("txt1", Selection::<'app>::TextArea(textarea1));
        map.insert("txt2", Selection::<'app>::TextArea(textarea2));

        let app = App {
            tag_matrix: vec![vec!["tab1", "tab2"], vec!["txt1", "txt2"]],
            tag_map: map,
            current_coo: (0, 1),
            last_tab_coo: (0, 0),
        };
        app
    }

    pub fn render<'render: 'app, B: Backend>(&mut self, f: &mut Frame<B>) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref());

        let tab_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref());
        let x = tab_layout.split(f.size());

        let tabs = self.tag_map.get("tab2").unwrap();
        if let Selection::Tab(_, tabs) = tabs {
            f.render_widget(tabs.borrow().clone(), x[0]);
        }

        let top_layout = vertical_layout.split(x[1])[0];
        let chunks = layout.split(top_layout);
        for (textarea_tag, chunk) in ["txt1", "txt2"].iter().zip(chunks) {
            let textarea = self.tag_map.get_mut(textarea_tag).unwrap().textarea();
            let widget = textarea.widget();
            f.render_widget(widget, chunk);
        }
    }

    fn select(&mut self, element_tag: &str) {
        let element = self.tag_map.get_mut(element_tag).unwrap();
        match element {
            Selection::TextArea(textarea) => App::_textarea_selected(textarea),
            Selection::Tab(tab_n, tabs) => {
                tabs.replace_with(|t| {
                    t.clone()
                        .select(*tab_n)
                        .style(Style::default().fg(Color::White))
                });
            }
        };
    }

    fn unselect(&mut self, element_tag: &str) {
        let element = self.tag_map.get_mut(element_tag).unwrap();
        match element {
            Selection::TextArea(textarea) => App::_textarea_unselected(textarea),
            Selection::Tab(_, tabs) => {
                tabs.replace_with(|t| t.clone().style(Style::default().fg(Color::DarkGray)));
            }
        }
    }

    pub fn input(&mut self, input: Input) {
        match input {
            Input {
                key: Key::Char('l'),
                ..
            } => self.move_selection(SelectionMove::Right),
            Input {
                key: Key::Char('h'),
                ..
            } => self.move_selection(SelectionMove::Left),
            Input {
                key: Key::Char('k'),
                ..
            } => self.move_selection(SelectionMove::Up),
            Input {
                key: Key::Char('j'),
                ..
            } => self.move_selection(SelectionMove::Down),

            _ => {} // if let Some(input) = tui_app::convert_jkhl_to_arrow(input) {
                    //     // textarea[current].input(input);
                    // }
        }
    }

    fn move_selection(&mut self, direction: SelectionMove) -> () {
        let (x, y) = self.current_coo.clone();
        let (n_x, n_y) = App::_calculate_new_coo(direction, (x, y), &self.tag_matrix);

        let txta = self.tag_map.get_mut("txt2").unwrap().textarea();
        txta.delete_line_by_end();
        txta.delete_line_by_head();
        txta.insert_str(format!("{:?} -> {:?}", (x, y), (n_x, n_y)));

        if (x, y) == (n_x, n_y) {
            return;
        }

        let (n_x, n_y) = match (y, n_y) {
            (1, 0) => self.last_tab_coo,
            (0, 1) => {
                self.last_tab_coo = (x, y);
                (0, n_y)
            }
            _ => (n_x, n_y),
        };

        self.unselect(&self.tag_matrix[y][x]);
        self.select(&self.tag_matrix[n_y][n_x]);

        self.current_coo = (n_x, n_y);
    }

    fn _calculate_new_coo(
        direciton: SelectionMove,
        (x, y): (usize, usize),
        matrix: &Vec<Vec<&str>>,
    ) -> (usize, usize) {
        // [
        //     [0,0 , 0,1]
        //     [0,1 , 1,1]
        // ]
        let (n_x, n_y) = match direciton {
            SelectionMove::Right if (x + 1) < matrix[y].len() => (x + 1, y),
            SelectionMove::Left if x > 0 => (x - 1, y),
            SelectionMove::Down if (y + 1) < matrix.len() => (x, y + 1),
            SelectionMove::Up if y > 0 => (x, y - 1),
            _ => (x, y),
        };
        (n_x, n_y)
    }

    fn _textarea_unselected(textarea: &mut TextArea<'_>) {
        textarea.set_cursor_line_style(Style::default());
        textarea.set_cursor_style(Style::default());
        let b = textarea
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        textarea.set_block(b.style(Style::default().fg(Color::DarkGray)));
    }

    fn _textarea_selected(textarea: &mut TextArea<'_>) {
        textarea.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));
        textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        let b = textarea
            .block()
            .cloned()
            .unwrap_or_else(|| Block::default().borders(Borders::ALL));
        textarea.set_block(b.style(Style::default()));
    }
}

fn convert_jkhl_to_arrow(input: Input) -> Option<Input> {
    match input {
        Input {
            key: Key::Char('j'),
            ..
        } => Some(Input {
            key: Key::Down,
            ..input
        }),
        Input {
            key: Key::Char('k'),
            ..
        } => Some(Input {
            key: Key::Up,
            ..input
        }),
        Input {
            key: Key::Char('h'),
            ..
        } => Some(Input {
            key: Key::Left,
            ..input
        }),
        Input {
            key: Key::Char('l'),
            ..
        } => Some(Input {
            key: Key::Right,
            ..input
        }),
        _ => None,
    }
}
