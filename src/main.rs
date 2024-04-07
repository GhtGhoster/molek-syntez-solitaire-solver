use std::{collections::HashSet, fmt::Display, hash::Hash, io, thread::sleep, time::Duration};
use colored::Colorize;
use enigo::{Enigo, MouseControllable};
use rand::{thread_rng, Rng};
use screenshots::{image::{io::Reader, GenericImageView, ImageBuffer}, Screen};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[allow(dead_code)]
const BOX_WIDTH: u32 = 22;
#[allow(dead_code)]
const BOX_HEIGHT: u32 = 18;
const OFFSET_H: i32 = 488;
const OFFSET_V: i32 = 298;
const SPACE_H: i32 = 652 - OFFSET_H;
const SPACE_V: i32 = 330 - OFFSET_V;

const STEP_LIMIT: usize = 2000;
const PAST_LIMIT: usize = 20000;
const ACCEPTABLE_SOLUTION_LEN: usize = 400;

#[derive(Clone, Copy, PartialEq, Debug, EnumIter)]
enum Card {
    Tits = 8,
    King = 7,
    Diva = 6,
    Viva = 5,
    Ten = 4,
    Nine = 3,
    Eight = 2,
    Seven = 1,
    Six = 0,
}

impl Card {
    fn from_char(value: char) -> Option<Self> {
        match value {
            't' | 'T' => Some(Card::Tits),
            'k' | 'K' => Some(Card::King),
            'd' | 'D' => Some(Card::Diva),
            'v' | 'V' => Some(Card::Viva),
            '0' | '1' => Some(Card::Ten),
            '9' => Some(Card::Nine),
            '8' => Some(Card::Eight),
            '7' => Some(Card::Seven),
            '6' => Some(Card::Six),
            _ => None,
        }
    }

    fn to_char(&self) -> char {
        let character = match self {
            Card::Tits => 'T',
            Card::King => 'K',
            Card::Diva => 'D',
            Card::Viva => 'V',
            Card::Ten => '0',
            Card::Nine => '9',
            Card::Eight => '8',
            Card::Seven => '7',
            Card::Six => '6',
        };
        character
    }
    
    fn from_image(image: ImageBuffer<screenshots::image::Rgba<u8>, Vec<u8>>) -> Card {
        'card_loop: for card_type in Card::iter() {
            let card_image = Reader::open(format!("assets/{}.png", card_type.to_char())).unwrap().decode().unwrap();
            for x in 0..BOX_WIDTH {
                for y in 0..BOX_HEIGHT {
                    if &card_image.get_pixel(x, y) != image.get_pixel(x, y) {
                        continue 'card_loop;
                    }
                }
            }
            return card_type;
        }
        panic!();
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Default, Clone)]
struct Stack {
    cards: Vec<Card>,
    collapsed: bool,
    cheated: bool,
}

impl Stack {
    fn highest_orderly_count(&self) -> usize {
        let len = self.cards.len();
        if len < 2 {
            return len;
        }
        let mut ret = 1;
        while self.cards[len-ret] as usize + 1 == self.cards[len-ret-1] as usize {
            ret += 1;
            if ret == self.cards.len() {
                return ret;
            }
        }
        ret
    }

    fn is_orderly(&self, card: Card) -> bool {
        if self.cards.len() == 0 {
            return true;
        }
        let last_stack_card_index = self.cards.len()-1;
        let last_stack_card_num = self.cards[last_stack_card_index] as usize;
        let card_num = card as usize;
        card_num + 1 == last_stack_card_num
    }
}

#[derive(PartialEq)]
enum MoveValidity {
    ValidNormal,
    ValidCheat,
    Invalid,
}

// 9 different types of cards, 4 sets of cards, 6 columns, 6 rows at the start, 36 cards total
#[derive(Default)]
struct Matrix {
    stacks: [Stack; 6],
    available_moves: Vec<(Move, Matrix)>,
    past_moves: Vec<Move>,
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Matrix {}

impl Hash for Matrix {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Matrix {
    #[allow(dead_code)]
    fn from_screen() -> Matrix {
        let mut matrix: Matrix = Default::default();

        // grab the screen. This is specifically set up for my use-case
        // aka 3 monitors at FHD, game running on middle monitor with whatever order I've set up
        let screens = Screen::all().unwrap();
        let screen = screens[0];

        for x in 0..6 {
            for y in 0..6 {
                let image: ImageBuffer<screenshots::image::Rgba<u8>, Vec<u8>> = screen.capture_area(
                    OFFSET_H + (x * SPACE_H),
                    OFFSET_V + (y * SPACE_V),
                    BOX_WIDTH,
                    BOX_HEIGHT,
                ).unwrap();
                // image.save("test.png").unwrap();
                // io::stdin().read_line(&mut String::new()).unwrap();
                let card = Card::from_image(image);
                matrix.stacks[x as usize].cards.push(card);
            }
        }

        matrix
    }

    #[allow(dead_code)]
    fn from_input() -> Matrix {
        let mut matrix: Matrix = Default::default();
    
        for _ in 0..6 {
            // handle and parse input
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            let mut parsed_line: Option<[Card; 6]> = parse_line(&buffer);
            buffer.clear();
    
            while parsed_line == Option::None {
                println!("Incorrect format, try again.");
                io::stdin().read_line(&mut buffer).unwrap();
                parsed_line = parse_line(&buffer);
                buffer.clear();
            }
    
            // structure input
            for (i, card) in parsed_line.unwrap().iter().enumerate() {
                matrix.stacks[i].cards.push(*card);
            }
        }

        // check input validity
        // Inefficient and potentially unsafe due to user-input based depth recursion, don't care tho.
        let mut type_count_array: [usize; 9] = [0; 9];
        for stack in &matrix.stacks {
            for card in &stack.cards {
                type_count_array[*card as usize] += 1;
            }
        }
        for i in 0..9 {
            if type_count_array[i] != 4 {
                println!("Wrong input parity, try again:");
                return Matrix::from_input();
            }
        }

        matrix
    }

    #[allow(dead_code)]
    fn random() -> Matrix {
        let mut matrix: Matrix = Default::default();
        let mut matrix_index = 0;
        let mut rng = thread_rng();

        for _ in 0..4 {
            let mut chars = "67890VDKT".to_string();
            while chars.len() != 0 {
                let random_index = rng.gen_range(0..chars.len());
                let character = chars.remove(random_index);
                matrix.stacks[matrix_index].cards.push(Card::from_char(character).unwrap());
                matrix_index += 1;
                matrix_index %= 6;
            }
        }

        matrix
    }

    fn check_validity(&self, move_: Move) -> MoveValidity {
        let Move{from, to, count} = move_;
        if
            self.stacks[to].collapsed ||
            self.stacks[from].collapsed ||
            self.stacks[to].cheated ||
            to == from
        {
            return MoveValidity::Invalid;
        }
        let first_card_index_from = self.stacks[from].cards.len() - count;
        if count == 1 {
            if self.stacks[to].is_orderly(self.stacks[from].cards[first_card_index_from]) {
                return MoveValidity::ValidNormal;
            }
            if self.stacks[from].cheated {
                return MoveValidity::Invalid;
            }
            return MoveValidity::ValidCheat;
        }
        if self.stacks[from].highest_orderly_count() > count {
            return MoveValidity::Invalid;
        }
        if self.stacks[to].cards.is_empty() {
            return MoveValidity::ValidNormal;
        }
        let last_card_index_to = self.stacks[to].cards.len() - 1;
        if self.stacks[to].cards[last_card_index_to] as usize == self.stacks[from].cards[first_card_index_from] as usize + 1 {
            return MoveValidity::ValidNormal;
        }
        return MoveValidity::Invalid;
    }

    fn move_stack(&mut self, move_: Move) -> bool {
        let Move{from, to, count} = move_;

        // validity check
        let validity: MoveValidity = self.check_validity(move_);
        if validity == MoveValidity::Invalid {
            return false;
        }
        if validity == MoveValidity::ValidCheat {
            self.stacks[to].cheated = true;
        }

        // actual movement
        let mut moving_cards: Vec<Card> =  vec![];

        self.stacks[from].cheated = false;
        for _ in 0..count {
            moving_cards.push(self.stacks[from].cards.pop().unwrap());
        }
        for _ in 0..count {
            self.stacks[to].cards.push(moving_cards.pop().unwrap());
        }

        // collapsed check
        if
            self.stacks[to].highest_orderly_count() == self.stacks[to].cards.len() &&
            self.stacks[to].cards.len() == 9
        {
            self.stacks[to].collapsed = true;
        }

        return true;
    }

    fn is_win(&self) -> bool {
        let mut collapsed_count = 0;
        for i in 0..6 {
            if self.stacks[i].collapsed {
                collapsed_count += 1;
            }
        }
        collapsed_count == 4
    }

    fn is_finished(&self) -> bool {
        self.valid_moves().is_empty()
    }

    fn valid_moves(&self) -> Vec<Move> {
        let mut ret = vec![];

        for from in 0..6 {
            let highest_orderly_count = self.stacks[from].highest_orderly_count();
            for count in 1..=highest_orderly_count {
                for to in 0..6 {
                    let move_: Move = Move {from, to , count};
                    if self.check_validity(move_) != MoveValidity::Invalid {
                        ret.push(move_);
                    }
                }
            }
        }

        ret
    }

    fn copy(&self) -> Matrix {
        let mut matrix: Matrix = Default::default();

        // copy
        for i in 0..6 {
            for j in 0..self.stacks[i].cards.len() {
                matrix.stacks[i].cards.push(self.stacks[i].cards[j]);
            }
            matrix.stacks[i].cheated = self.stacks[i].cheated;
            matrix.stacks[i].collapsed = self.stacks[i].collapsed;
        }
        for i in 0..self.past_moves.len() {
            matrix.past_moves.push(self.past_moves[i]);
        }

        matrix
    }

    fn copy_after_move(&self, move_: Move) -> Matrix {
        let mut matrix: Matrix = self.copy();
        matrix.move_stack(move_);
        matrix.past_moves.push(move_);
        matrix
    }

    fn to_string(&self) -> String {
        let mut ret = String::new();

        for stack in &self.stacks {
            ret.push('S');
            for card in &stack.cards {
                ret.push(card.to_char());
            }
            if stack.cheated {
                ret.push('_');
            }
        }

        ret
    }

    fn save_moves(&mut self) {
        self.available_moves = self
            .valid_moves()
            .iter()
            .map(|move_| (*move_, self.copy_after_move(*move_)))
            .collect();
        // prioritize low move count
        self.available_moves.sort_by(|(_, a), (_, b)| a.valid_moves().len().cmp(&b.valid_moves().len()));
    }

    fn prune(&mut self, past_matrices: &HashSet<Matrix>) {
        for i in (0..self.available_moves.len()).rev() {
            if past_matrices.contains(&self.available_moves[i].1) {
                self.available_moves.remove(i);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Move {
    from: usize,
    to: usize,
    count: usize,
}

impl Move {
    fn from_input() -> Move {
        let mut buffer: String = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        let tokens: Vec<&str> = buffer.trim().split(" ").collect();
        let from: usize = tokens[0].parse().unwrap();
        let to: usize = tokens[1].parse().unwrap();
        let count: usize = tokens[2].parse().unwrap();
        Move {
            from,
            to,
            count
        }
    }
}

fn main() {
    // TODO:
    // - improve found solutions via past matrices, cutting out middle parts (the end-game is atrocious due to most moves being preferred)
    // - automate the new game repetition process, if no short enough solution can be found, just new game it

    let mut matrix = Matrix::from_screen();
    // let mut matrix = Matrix::random();
    // let mut matrix = Matrix::from_input();
    
    // print_matrix(&matrix);
    let mut winners: Vec<Matrix> = vec![];
    let mut past_matrices: HashSet<Matrix> = HashSet::new();
    while past_matrices.len() < PAST_LIMIT {
        if let Some(winner) = find_win(&mut matrix, &mut past_matrices) {
            winners.push(winner);
        }
    }
    if winners.is_empty() {
        println!("No winzies :c");
    } else {
        winners.sort_by(|a, b| a.past_moves.len().cmp(&b.past_moves.len()));
        println!("Bestest winner: {}", winners[0].past_moves.len());
        execute_moves(&mut matrix, &winners[0].past_moves);
    }

    // gameplay_loop(&mut matrix);
}

fn loop_wins() {
    loop {
        // focus window, click new game, wait
        let mut matrix = Matrix::from_screen();
        
        let mut winners: Vec<Matrix> = vec![];
        let mut past_matrices: HashSet<Matrix> = HashSet::new();
        while past_matrices.len() < PAST_LIMIT {
            if let Some(winner) = find_win(&mut matrix, &mut past_matrices) {
                winners.push(winner);
            }
        }
        if !winners.is_empty() {
            winners.sort_by(|a, b| a.past_moves.len().cmp(&b.past_moves.len()));
            if winners[0].past_moves.len() > ACCEPTABLE_SOLUTION_LEN {
                continue;
            }
            execute_moves(&mut matrix, &winners[0].past_moves);
        }
    }
}

#[allow(dead_code)]
fn execute_moves(matrix: &mut Matrix, moves: &Vec<Move>) {
    let eta = moves.len() * (100 + 50 + 50 + 50) as usize;
    // println!("Estimated time: {} seconds, continue? [(y)/n]", eta as f32 / 1000.0);
    // let mut buf = String::new();
    // io::stdin().read_line(&mut buf).unwrap();
    // if buf.trim() == "n".to_string() {
    //     return;
    // }

    let mut enigo = Enigo::new();

    // focus window but don't pick a card if window already focused
    enigo.mouse_move_to(
        1920 + OFFSET_H - SPACE_H,
        OFFSET_V - SPACE_V,
    );
    sleep(Duration::from_millis(100));
    enigo.mouse_down(enigo::MouseButton::Left);
    sleep(Duration::from_millis(50));
    enigo.mouse_up(enigo::MouseButton::Left);
    sleep(Duration::from_millis(100));

    for mov in moves {
        let y_from = matrix.stacks[mov.from].cards.len() - mov.count;
        enigo.mouse_move_to(
            1920 + OFFSET_H + (mov.from as i32 * SPACE_H),
            OFFSET_V + (y_from.max(0) as i32 * SPACE_V),
        );
        enigo.mouse_down(enigo::MouseButton::Left);
        sleep(Duration::from_millis(50));
        enigo.mouse_up(enigo::MouseButton::Left);

        sleep(Duration::from_millis(50));

        let y_to = (matrix.stacks[mov.to].cards.len() as i32 - 1).max(0) as usize;
        enigo.mouse_move_to(
            1920 + OFFSET_H + (mov.to as i32 * SPACE_H),
            OFFSET_V + (y_to as i32 * SPACE_V),
        );
        enigo.mouse_down(enigo::MouseButton::Left);
        sleep(Duration::from_millis(50));
        enigo.mouse_up(enigo::MouseButton::Left);

        sleep(Duration::from_millis(100));

        matrix.move_stack(*mov);
    }
}

#[allow(dead_code)]
fn find_win(matrix: &mut Matrix, past_matrices: &mut HashSet<Matrix>) -> Option<Matrix> {
    past_matrices.insert(matrix.copy());
    if past_matrices.len() > PAST_LIMIT {
        return None;
    }

    matrix.save_moves();
    matrix.prune(&past_matrices);

    if matrix.available_moves.is_empty() {
        if matrix.is_win() {
            return Some(matrix.copy());
        } else {
            return None;
        }
    } else {
        if matrix.past_moves.len() > STEP_LIMIT {
            return None;
        }
        for i in 0..matrix.available_moves.len() {
            let result = find_win(&mut matrix.available_moves[i].1, past_matrices);
            if result.is_none() {
                continue;
            } else {
                return result;
            }
        }
    }
    None
}

#[allow(dead_code)]
fn gameplay_loop(matrix: &mut Matrix) {
    println!("{:?}", matrix.valid_moves());
    while !matrix.is_finished() {
        print_matrix(&matrix);
        let move_: Move = Move::from_input();
        if matrix.move_stack(move_) {
            println!("You done did good moved {} cards [{}] -> [{}]", move_.count, move_.from, move_.to);
        } else {
            println!("You done fucked goofed mister goober\n");
        }
    }
    if matrix.is_win() {
        println!("You's a winzies!!1!:D");
    } else {
        println!("You loozies :,ccc");
    }
}

#[allow(dead_code)]
fn print_matrix(matrix: &Matrix) {
    println!("======");
    let mut row = 0;
    loop {
        let mut should_break = true;
        for i in 0..6 {
            let stack = &matrix.stacks[i];
            if !stack.collapsed {
                if stack.cards.len() > row {
                    should_break = false;
                    let cheated: bool = stack.cards.len() == row + 1 && stack.cheated;
                    print!(
                        "{}",
                        if cheated {
                            format!("{}", stack.cards[row]).bold().blue()
                        } else {
                            format!("{}", stack.cards[row]).bold().clear()
                        }
                    );
                } else {
                    print!(" ");
                }
            } else {
                if row == 0 {
                    print!("{}", "C".bold().red());
                    should_break = false;
                } else {
                    print!(" ");
                }
            }
        }
        row += 1;
        println!();
        if should_break {
            break;
        }
    }
}

#[allow(dead_code)]
fn parse_line(string: &String) -> Option<[Card; 6]> {
    let mut ret: [Card; 6] = [Card::Six; 6];
    let tmp = string.trim();
    if tmp.len() != 6 {
        return None;
    }
    for (i, character) in tmp.chars().enumerate() {
        match Card::from_char(character) {
            Some(symbol) => {
                ret[i] = symbol;
            }
            None => {
                return None;
            }
        }
    }
    Some(ret)
}
