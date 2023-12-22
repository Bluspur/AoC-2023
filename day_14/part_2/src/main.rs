use std::collections::HashMap;

use anyhow::{Context, Result};
use rayon::prelude::*;

fn main() {
    let input = std::fs::read_to_string("./puzzle_input.txt").expect("Unable to read file");
    let result = solve_part(&input, 1000000000);
    println!("Result: {}", result);
}

fn solve_part(input: &str, cycles: usize) -> usize {
    let mut parsed_input = parse(input).expect("Failed to parse input");
    let mut seen: HashMap<_, _> = HashMap::new();
    let mut cycle_length = 0;
    let mut cycle_start = 0;

    for i in 0..cycles {
        if let Some(prev_i) = seen.get(&parsed_input) {
            cycle_start = *prev_i;
            cycle_length = i - cycle_start;
            break;
        }
        seen.insert(parsed_input.clone(), i);
        for _ in 0..4 {
            parsed_input = parsed_input.into_par_iter().map(slide_rocks).collect();
            parsed_input = rotate_2d_vector_clockwise(parsed_input);
        }
    }

    if cycle_length > 0 {
        let remaining_cycles = (cycles - cycle_start) % cycle_length;
        for _ in 0..remaining_cycles {
            for _ in 0..4 {
                parsed_input = parsed_input.into_par_iter().map(slide_rocks).collect();
                parsed_input = rotate_2d_vector_clockwise(parsed_input);
            }
        }
    }

    parsed_input
        .into_par_iter()
        .map(|col| calculate_load(&col))
        .sum()
}

fn rotate_2d_vector_clockwise(vector: Vec<Vec<PositionState>>) -> Vec<Vec<PositionState>> {
    let n = vector.len();
    let m = vector[0].len();
    let mut result = vec![vec![PositionState::Empty; n]; m];
    for i in 0..n {
        for j in 0..m {
            result[j][n - i - 1] = vector[i][j];
        }
    }
    result
}

fn parse(input: &str) -> Result<Vec<Vec<PositionState>>> {
    let row_len = input
        .find(char::is_whitespace)
        .context("Input should be seperated with line breaks")?;

    let cleaned_input: Vec<_> = input
        .chars()
        .filter(|c| matches!(c, 'O' | '.' | '#'))
        .collect();

    let columns: Vec<_> = (0..row_len)
        .into_par_iter()
        .map(|i| {
            let mut column: Vec<_> = cleaned_input
                .iter()
                .enumerate()
                .filter(|(j, _)| j % row_len == i)
                .map(|(_, &c)| match c {
                    'O' => PositionState::RoundRock,
                    '.' => PositionState::Empty,
                    '#' => PositionState::CubeRock,
                    _ => unreachable!(),
                })
                .collect();
            column.reverse();
            column
        })
        .collect();

    Ok(columns)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum PositionState {
    RoundRock,
    CubeRock,
    Empty,
}

fn slide_rocks(mut positions: Vec<PositionState>) -> Vec<PositionState> {
    // We know that our positions should not be empty
    assert!(!positions.is_empty());
    // We start counting from the end to enable easier cascading
    let mut current_position = positions.len();
    // Initialise our blocker pointer to None
    let mut last_available_space = None;

    // Loop through our positions excluding the last position
    for _ in 0..positions.len() {
        current_position -= 1;
        let position_state = positions[current_position];

        if let Some(last_space_index) = last_available_space {
            // A space to slide is available
            match position_state {
                PositionState::RoundRock => {
                    positions[last_space_index] = PositionState::RoundRock;
                    positions[current_position] = PositionState::Empty;
                    last_available_space = Some(last_space_index - 1);
                }
                PositionState::CubeRock => last_available_space = None,
                PositionState::Empty => {}
            }
        } else {
            // Nowhere to slide, so we only care about empty spaces
            if position_state == PositionState::Empty {
                last_available_space = Some(current_position);
            }
        }
    }

    positions
}

fn calculate_load(positions: &[PositionState]) -> usize {
    positions
        .iter()
        .enumerate()
        .fold(0, |acc, (position, state)| {
            if state == &PositionState::RoundRock {
                acc + position + 1
            } else {
                acc
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_solve_part() {
        let input = indoc! {"
        O....#....
        O.OO#....#
        .....##...
        OO.#O....O
        .O.....O#.
        O.#..O.#.#
        ..O..#O..O
        .......O..
        #....###..
        #OO..#....
        "};
        assert_eq!(solve_part(input, 1000000000), 64);
    }

    #[test]
    fn test_parse_input() {
        let input = indoc! {"
        O.#
        #..
        .O#
        "};

        let expected = vec![
            vec![
                PositionState::Empty,
                PositionState::CubeRock,
                PositionState::RoundRock,
            ],
            vec![
                PositionState::RoundRock,
                PositionState::Empty,
                PositionState::Empty,
            ],
            vec![
                PositionState::CubeRock,
                PositionState::Empty,
                PositionState::CubeRock,
            ],
        ];

        let actual = parse(input).expect("Testing input should not fail to parse");

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_rocks_slide_to_correct_positions() {
        let line = vec![
            PositionState::RoundRock,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::CubeRock,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
        ];
        let expected = vec![
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::RoundRock,
            PositionState::CubeRock,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
        ];

        let actual = slide_rocks(line);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_calculate_load_returns_correct_value() {
        let line = vec![
            PositionState::RoundRock,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::CubeRock,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
            PositionState::Empty,
        ];
        // Before Sliding
        let expected = 1;
        let actual = calculate_load(&line);
        assert_eq!(actual, expected);
        // After Sliding
        let expected = 5;
        let actual = slide_rocks(line);
        let actual = calculate_load(&actual);
        assert_eq!(actual, expected);
    }
}
