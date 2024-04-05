use std::{fmt::Display, io};
use colored::Colorize;
use rand::{thread_rng, Rng};

#[derive(Clone, Copy, PartialEq, Debug)]
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
        println!("{:?}", self.cards);
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
}

impl Matrix {
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
        match validity {
            MoveValidity::ValidNormal => {
                if
                    self.stacks[to].highest_orderly_count() == self.stacks[to].cards.len() &&
                    self.stacks[to].cards.len() == 9
                {
                    self.stacks[to].collapsed = true;
                }
            },
            MoveValidity::ValidCheat => {
                self.stacks[to].cheated = true;
            },
            MoveValidity::Invalid => {
                return false;
            },
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

    fn is_lose(&self) -> bool {
        !self.valid_moves().is_empty()
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
}

#[derive(Clone, Copy)]
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
    // - copy matrix
    // - matrix hash function
    // - tree exploration
    // - heuristics
    //      - minimize cheating
    //      - sort by most available moves (maybe cache those)
    //      - account for reversible moves (and remove them, here cache would also be useful)


    #[allow(unused)]
    let mut matrix = Matrix::random();
    // let mut matrix = Matrix::from_input();

    // matrix.stacks[0].cheated = true;
    // matrix.stacks[2].collapsed = true;
    // matrix.move_stack(0, 5, 2);

    // println!();
    // print_matrix(&matrix);
    
    gameplay_loop(&mut matrix);
}

#[allow(dead_code)]
fn gameplay_loop(matrix: &mut Matrix) {
    while !matrix.is_win() && !matrix.is_lose() {
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
    }
    if matrix.is_lose() {
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
