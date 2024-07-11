use crate::{
    app::UiBlock,
    schema::{Block, FilenameParseError},
};
use rust_fsm::{StateMachine, StateMachineImpl};

struct Parser {}

impl Parser {
    fn parse(
        input: String,
        blocks: Block,
        delim: String,
    ) -> Result<Vec<UiBlock>, FilenameParseError> {
        let mut output = None;
        let mut dfa = StateMachine::<Parser>::new();
        let input = Input {
            input,
            blocks,
            delim,
        };
        // run the state machine to a final state
        while output.is_none() {
            output = dfa.consume(&input).unwrap();
        }
        output.unwrap()
    }
}

struct Input {
    input: String,
    blocks: Block,
    delim: String,
}

struct State {
    i: usize,

    blocks: Vec<UiBlock>,
}

impl StateMachineImpl for Parser {
    type Input = Input;
    type State = State;
    type Output = Result<Vec<UiBlock>, FilenameParseError>;

    const INITIAL_STATE: Self::State = State {
        i: 0,
        blocks: vec![],
    };

    fn transition(state: &Self::State, input: &Self::Input) -> Option<Self::State> {
        todo!()
    }

    fn output(state: &Self::State, input: &Self::Input) -> Option<Self::Output> {
        if state.i != input.input.len() {
            Some(Err(FilenameParseError::UnexpectedTag(
                input.input[state.i..].to_string(),
            )))
        } else {
        }
    }
}
