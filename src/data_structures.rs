//! Data structures used by the Teko library

use std::collections::HashMap;
use std::rc::Rc;

use num::BigInt;

use user::Userdata;

/// Evaluation commands used internally by the interpreter
///
/// When put on the stack these values have different effects on the interpreter.
#[derive(Debug, PartialEq)]
pub enum Commands {
	Call(Statement), // Should this be arbitrary data? Change to Transfer or macro!
	// Cmo(Macro),
	// Cfn(Function),
	Prep(Statement), // Should be arbitrary data! But shorten to Prep
	Param, // Shorten to Param
	Depar(Vec<String>), // Shorten to Depar
	If(Statement, Statement),
	Wind,
	Eval, // Shorten to Eval
}

/// Top level data structure used by the parser and interpreter
#[derive(Debug)]
pub struct Sourcedata(pub Option<Source>, pub Coredata);
/// Top level statements are reference counted `Sourcedata`
pub type Statement = Rc<Sourcedata>;
/// A program is an ordered sequence of `Statement`
pub type Program = Vec<Statement>;

/// Denotes a "transfer function" that transform the state of the program
///
pub type Transfer = fn(program: &mut Program, env: &mut Env) -> Option<(Option<Source>, String)>;
/// Boolean values
#[derive(Debug, PartialEq)]
pub enum Boolean {
	True,
	False,
}

/// Function types that can be called by the interpreter
pub enum Function {
	/// A function written in the implementation language
	Builtin(Transfer, String),
	/// Parameter names with a sequence of statements that are inserted into the program when called
	Library(Vec<String>, Program),
}

/// Macro types that can be called by the interpreter
pub enum Macro {
	/// A function written in the implementation language
	Builtin(Transfer, String),
	/// Parameter name with a sequence of statements that are inserted into the program when called
	Library(String, Program),
}

/// Core data types of the Teko machine
#[derive(Debug)]
pub enum Coredata {
	// TODO Add complex number and representation symbol evaluation
	// TODO Add quoted form for writing out whatever in plain
	/// Denote true and false
	Boolean(bool),
	/// A pair of data items
	Cell(Rc<Sourcedata>, Rc<Sourcedata>), // TODO Ensure Non-nestedness
	/// Error type
	Error(Statement),  // Really necessary?
	/// Function type
	Function(Function), // OK
	/// Integer numbers
	Integer(BigInt), // Ok
	/// Internal commands (used by the implementation)
	Internal(Commands), // Fine,... I guess... but want a different enum for this
	/// Macro types
	Macro(Macro), 
	/// Null (an empty list)
	Null(), // Do we need this? null Yeah, () is core data
	/// String type
	String(String),
	/// Symbol type
	Symbol(String),

	/// User defined data
	User(Userdata), // Should remove this, unnecessary. All other data can be represented here
}

/// Environment used by the implementation
pub struct Env {
	/// Maps variables to stacks of variables (Program)
	pub store: HashMap<String, Program>,
	/// Parameter stack used for function calls
	pub params: Vec<Program>,
	/// Register used to store results of previous computations
	pub result: Statement,
}

/// State used by the parser internally
#[derive(Clone, Debug)]
pub struct ParseState {
	/// Most recent position in the stream being read
	pub current_read_position: Source,
	/// Last position where the beginning of a lexeme was initiated
	pub start_of_current_lexeme: Source,
	/// Stack of yet unmatched opening parentheses
	pub unmatched_opening_parentheses: Vec<Source>,
	/// The current lexeme being built into a token
	pub token: String,
	/// The stack of lists being built.
	pub stack: Vec<Program>,
	/// Error container, set to Some if the parser fails
	pub error: Option<String>,
}

/// Information about the source of data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
	/// Line number of the input, starts at 1
	pub line: usize,
	/// Column number of the input, starts at 1
	pub column: usize,
	/// Free-form string describing the source ("tty" from terminal, filename from file,..)
	pub source: String, // Change to Rc<String>
}
