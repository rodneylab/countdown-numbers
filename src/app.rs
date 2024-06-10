use std::collections::HashMap;

use rand::{prelude::SliceRandom, rngs::ThreadRng, thread_rng, Rng};

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    Introduction,

    PickingNumbers,
    Playing,
    DisplayingResult,
}

const LARGE_NUMBER_COUNT: usize = 4;
const SMALL_NUMBER_COUNT: usize = 20;

#[derive(Debug, Default)]
pub struct App {
    pub current_screen: CurrentScreen,
    pub available_small_numbers: [Option<u32>; SMALL_NUMBER_COUNT],
    pub available_large_numbers: [Option<u32>; LARGE_NUMBER_COUNT],
    pub selected_numbers: [Option<u32>; 6],
    pub target: u32,
    pub value_input: String,
    pub feedback: String,
    rng: ThreadRng,
}

impl App {
    pub fn new() -> App {
        let mut rng = thread_rng();

        // generate random large numbers
        let mut available_large_numbers = [25, 50, 75, 100];
        available_large_numbers.shuffle(&mut rng);
        let available_large_numbers = available_large_numbers.map(Some);

        // generate random small numbers
        let mut available_small_numbers =
            [1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10];
        available_small_numbers.shuffle(&mut rng);
        let available_small_numbers = available_small_numbers.map(Some);

        App {
            current_screen: CurrentScreen::Introduction,
            available_small_numbers,
            available_large_numbers,
            selected_numbers: [None; 6],
            target: rng.gen_range(100..1_000),
            rng,
            value_input: String::new(),
            feedback: String::new(),
        }
    }

    fn random_available_large_number_index(&mut self) -> Option<usize> {
        if !self
            .available_large_numbers
            .iter()
            .any(std::option::Option::is_some)
        {
            return None;
        }

        loop {
            let index = self.rng.gen_range(0..LARGE_NUMBER_COUNT);

            if self.available_large_numbers[index].is_some() {
                return Some(index);
            }
        }
    }

    pub fn is_number_selection_complete(&self) -> bool {
        !self
            .selected_numbers
            .iter()
            .any(std::option::Option::is_none)
    }

    fn random_available_small_number_index(&mut self) -> Option<usize> {
        if !self
            .available_small_numbers
            .iter()
            .any(std::option::Option::is_some)
        {
            return None;
        }

        loop {
            let index = self.rng.gen_range(0..SMALL_NUMBER_COUNT);

            if self.available_small_numbers[index].is_some() {
                return Some(index);
            }
        }
    }

    pub fn pick_random_large_number(&mut self) {
        if let Some(index_value) = self.random_available_large_number_index() {
            let result = self.available_large_numbers[index_value];
            let picked_index = self.selected_numbers.iter().position(|&val| val.is_none());
            if let Some(picked_index_value) = picked_index {
                if result.is_some() {
                    self.selected_numbers[picked_index_value] = result;
                    self.available_large_numbers[index_value] = None;
                };
            }
        }
    }

    pub fn pick_random_small_number(&mut self) {
        if let Some(index_value) = self.random_available_small_number_index() {
            let result = self.available_small_numbers[index_value];
            let picked_index = self.selected_numbers.iter().position(|&val| val.is_none());
            if let Some(picked_index_value) = picked_index {
                if result.is_some() {
                    self.selected_numbers[picked_index_value] = result;
                    self.available_small_numbers[index_value] = None;
                };
            }
        }
    }

    pub fn check_solution(&self) -> Option<u32> {
        let input = self.value_input.trim();

        if input.trim().is_empty() {
            return None;
        }

        let solution_numbers = get_solution_numbers(input);

        if solution_numbers.len() > 6 {
            return None;
        }

        if !check_solution_numbers(&solution_numbers, &self.selected_numbers) {
            return None;
        }

        check_solution_calculation(input, self.target)
    }
}

fn check_solution_calculation(solution: &str, target: u32) -> Option<u32> {
    if let Ok(calculation_value) = num_parser::eval(solution) {
        let calculation_value: u32 = calculation_value
            .as_int()
            .expect("Should be able to represent calculation result as an integer")
            .try_into()
            .expect("Should be able to represent calculation result as a64-bit integer");
        if calculation_value > target {
            return Some(calculation_value - target);
        }
        return Some(target - calculation_value);
    }
    None
}

fn check_solution_numbers(solution_numbers: &[u32], selected_numbers: &[Option<u32>; 6]) -> bool {
    let unused_number_values: [u32; 6] = selected_numbers.map(|val| {
        val.expect("Solution should be checked against complete set of selected numbers")
    });

    // Build frequency map of selected numbers
    let mut unused_numbers: HashMap<u32, u32> =
        unused_number_values
            .iter()
            .fold(HashMap::new(), |mut accum, val| {
                accum.entry(*val).and_modify(|freq| *freq += 1).or_insert(1);
                accum
            });

    // Remove matching instances from frequency map matching solution numbers
    for number in solution_numbers {
        match unused_numbers.get(number) {
            None => return false,
            Some(1) => {
                unused_numbers.remove(number);
            }
            Some(_) => {
                unused_numbers.entry(*number).and_modify(|val| *val -= 1);
            }
        }
    }
    true
}

fn get_solution_numbers(solution: &str) -> Vec<u32> {
    let result =
        solution
            .split(|c: char| !c.is_ascii_digit())
            .fold(Vec::<u32>::new(), |mut accum, val| {
                if !val.is_empty() {
                    if let Ok(value) = val.parse::<u32>() {
                        accum.push(value);
                    };
                }
                accum
            });
    result
}

#[cfg(test)]
mod tests {
    use super::{
        check_solution_calculation, check_solution_numbers, get_solution_numbers, App,
        LARGE_NUMBER_COUNT, SMALL_NUMBER_COUNT,
    };

    #[test]
    fn random_available_large_number_index_returns_only_valid_index_as_expected() {
        // arrange
        let mut app = App::new();
        app.available_large_numbers[0] = None;
        app.available_large_numbers[1] = None;
        app.available_large_numbers[3] = None;

        // act
        let result = app.random_available_large_number_index();

        // assert
        assert_eq!(result, Some(2));
    }

    #[test]
    fn random_available_large_number_index_returns_valid_index_as_expected() {
        // arrange
        let mut app = App::new();

        // act
        for _ in 0..LARGE_NUMBER_COUNT {
            let index = app.random_available_large_number_index();
            app.available_large_numbers[index.unwrap()] = None;
        }

        let result = app
            .available_large_numbers
            .iter()
            .any(std::option::Option::is_some);

        // assert
        assert!(!result);
    }

    #[test]
    fn random_available_large_number_index_returns_none_as_expected() {
        // arrange
        let mut app = App::new();
        for _ in 0..LARGE_NUMBER_COUNT {
            let index = app.random_available_large_number_index();
            app.available_large_numbers[index.unwrap()] = None;
        }

        // act
        let result = app.random_available_large_number_index();

        // assert
        assert_eq!(result, None);
    }

    #[test]
    fn random_available_small_number_index_returns_none_as_expected() {
        // arrange
        let mut app = App::new();
        for _ in 0..SMALL_NUMBER_COUNT {
            let index = app.random_available_small_number_index();
            app.available_small_numbers[index.unwrap()] = None;
        }

        // act
        let result = app.random_available_small_number_index();

        // assert
        assert_eq!(result, None);
    }

    #[test]
    fn is_number_selection_complete_returns_false_as_expected() {
        // arrange
        let mut app = App::new();

        // act
        let result = app.is_number_selection_complete();

        // assert
        assert!(!result);

        // arrange
        app.selected_numbers[0] = Some(1);
        app.selected_numbers[1] = Some(1);

        // act
        let result = app.is_number_selection_complete();

        // assert
        assert!(!result);
    }

    #[test]
    fn is_number_selection_complete_returns_true_as_expected() {
        // arrange
        let mut app = App::new();
        app.pick_random_large_number();
        app.pick_random_large_number();
        app.pick_random_large_number();
        app.pick_random_small_number();
        app.pick_random_small_number();
        app.pick_random_small_number();

        // act
        let result = app.is_number_selection_complete();

        // assert
        assert!(result);
    }

    #[test]
    fn check_solution_returns_none_for_empty_solution() {
        // arrange
        let mut app = App::new();
        app.value_input = String::from("  ");

        // act
        let result = app.check_solution();

        assert_eq!(result, None);

        // arrange
        app.value_input = String::new();

        // act
        let result = app.check_solution();

        // assert
        assert_eq!(result, None);
    }

    #[test]
    fn check_solution_returns_none_for_solution_using_non_selected_numbers() {
        // arrange
        let mut app = App::new();
        app.selected_numbers = [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)];
        app.value_input = String::from("(1 + 2 + 3 + 4 + 5 + 6) * 7");
        app.target = 147;

        // act
        let result = app.check_solution();

        // assert
        assert_eq!(result, None);
    }

    #[test]
    fn check_solution_returns_none_for_solution_using_selected_number_too_many_times() {
        // arrange
        let mut app = App::new();
        app.selected_numbers = [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)];
        app.value_input = String::from("1 + 2 + 3 + 4 + 5 + 5");
        app.target = 20;

        // act
        let result = app.check_solution();

        // assert
        assert_eq!(result, None);
    }

    #[test]
    fn check_solution_returns_expected_result_if_not_all_numbers_used() {
        // arrange
        let mut app = App::new();
        app.selected_numbers = [Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)];
        app.value_input = String::from("1 + 2 + 3 + 4 + 5");
        app.target = 15;

        // act
        let result = app.check_solution();

        // assert
        assert_eq!(result, Some(0));
    }

    #[test]
    fn get_solution_parses_valid_input() {
        // arrange
        let input = "(10 *2) + 3 - 2 / 1";

        // act
        let result = get_solution_numbers(input);

        // assert
        assert_eq!(result, [10, 2, 3, 2, 1]);
    }

    #[test]
    fn check_solution_numbers_identifies_correct_numbers() {
        // arrange
        let input_numbers = [10, 2, 3, 2, 1];
        let selected_numbers = [Some(10), Some(2), Some(3), Some(2), Some(1), Some(75)];

        // act
        let result = check_solution_numbers(&input_numbers, &selected_numbers);

        // assert
        assert!(result);
    }

    #[test]
    fn check_solution_numbers_identifies_incorrect_repeated_numbers() {
        // arrange
        let input_numbers = [10, 2, 3, 2, 2];
        let selected_numbers = [Some(10), Some(2), Some(3), Some(2), Some(1), Some(75)];

        // act
        let result = check_solution_numbers(&input_numbers, &selected_numbers);

        // assert
        assert!(!result);
    }

    #[test]
    fn check_solution_numbers_identifies_incorrect_absent_numbers() {
        // arrange
        let input_numbers = [9, 2, 3, 2, 1];
        let selected_numbers = [Some(10), Some(2), Some(3), Some(2), Some(1), Some(75)];

        // act
        let result = check_solution_numbers(&input_numbers, &selected_numbers);

        // assert
        assert!(!result);
    }

    #[test]
    fn check_solution_calculation_parses_valid_input() {
        // arrange
        let input = "(10 * 2) + 3 - 2 / 1";

        // act
        let result = check_solution_calculation(input, 21);

        // assert
        assert_eq!(result, Some(0));
    }

    #[test]
    fn check_solution_calculation_returns_expected_value_for_large_calculation() {
        // arrange
        let input = "(10 * 2) + 3 - 2 / 1";

        // act
        let result = check_solution_calculation(input, 20);

        // assert
        assert_eq!(result, Some(1));
    }

    #[test]
    fn check_solution_calculation_returns_expected_value_for_small_calculation() {
        // arrange
        let input = "(10 * 2) + 3 - 2 / 1";

        // act
        let result = check_solution_calculation(input, 22);

        // assert
        assert_eq!(result, Some(1));
    }

    #[test]
    fn check_solution_calculation_parses_invalid_input() {
        // arrange
        let input = "(10 * 2 + 3 - 2 / 1";

        // act
        let result = check_solution_calculation(input, 21);

        // assert
        assert_eq!(result, None);
    }
}
