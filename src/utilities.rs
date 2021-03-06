//! Utilities used by the implementation.

use std::{cmp, convert, fmt, sync::Arc, usize};

use data_structures::*;
use super::VEC_CAPACITY;

pub fn program_to_cells(program: &Program) -> Statement {
	let mut top = rcs(Coredata::Null());
	for i in program {
		top = rcs(Coredata::Cell(i.clone(), top.clone()));
	}
	top
}

// //////////////////////////////////////////////////////////
// Impls
// //////////////////////////////////////////////////////////

impl cmp::PartialEq for Coredata {
	fn eq(&self, other: &Self) -> bool {
		if self as *const Coredata == other as *const Coredata {
			return true;
		}
		match *self {
			Coredata::Boolean(true) => {
				if let Coredata::Boolean(true) = *other {
					true
				} else {
					false
				}
			}
			Coredata::Boolean(false) => {
				if let Coredata::Boolean(false) = *other {
					true
				} else {
					false
				}
			}
			Coredata::Error(ref lhs) => {
				if let Coredata::Error(ref rhs) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Function(Function::Builtin(_, ref lhs)) => {
				if let Coredata::Function(Function::Builtin(_, ref rhs)) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Function(Function::Library(ref lhsparams, ref lhscode)) => {
				if let Coredata::Function(Function::Library(ref rhsparams, ref rhscode)) = *other {
					lhsparams == rhsparams && lhscode == rhscode
				} else {
					false
				}
			}
			Coredata::Integer(ref lhs) => {
				if let Coredata::Integer(ref rhs) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Macro(Macro::Builtin(_, ref lhs)) => {
				if let Coredata::Macro(Macro::Builtin(_, ref rhs)) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Macro(Macro::Library(ref lhsparam, ref lhscode)) => {
				if let Coredata::Macro(Macro::Library(ref rhsparam, ref rhscode)) = *other {
					lhsparam == rhsparam && lhscode == rhscode
				} else {
					false
				}
			}
			Coredata::Internal(ref lhs) => {
				if let Coredata::Internal(ref rhs) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Null() => {
				if let Coredata::Null() = *other {
					true
				} else {
					false
				}
			}
			Coredata::Cell(ref lhshead, ref lhstail) => {
				if let Coredata::Cell(ref rhshead, ref rhstail) = *other {
					lhshead == rhshead && lhstail == rhstail
				} else {
					false
				}
			}
			Coredata::String(ref lhs) => {
				if let Coredata::String(ref rhs) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Symbol(ref lhs) => {
				if let Coredata::Symbol(ref rhs) = *other {
					lhs == rhs
				} else {
					false
				}
			}
			Coredata::Table(ref lhs) => {
				if let Coredata::Table(ref rhs) = *other {
					lhs == rhs
				} else {
					false
				}
			}
		}
	}
}

impl cmp::PartialEq for Sourcedata {
	fn eq(&self, other: &Self) -> bool {
		self.1 == other.1
	}
}

impl fmt::Debug for Function {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Function::Builtin(.., ref name) => {
				write![f, "{}", name]?;
			}
			Function::Library(ref params, ref code) => {
				write![f, "(fn ("]?;
				let mut first = true;
				for i in params.iter() {
					if first {
						write![f, "{:?}", i]?;
					} else {
						write![f, " {:?}", i]?;
					}
					first = false;
				}
				write![f, ")"]?;
				for i in code.iter().rev() {
					write![f, " {:?}", i]?;
				}
				write![f, ")"]?;
			}
		}
		Ok(())
	}
}

impl fmt::Debug for Macro {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Macro::Builtin(.., ref name) => {
				write![f, "{}", name]?;
			}
			Macro::Library(ref param, ref code) => {
				write![f, "(macro {:?}", param]?;
				for i in code.iter().rev() {
					write![f, " {}", i]?;
				}
				write![f, ")"]?;
			}
		}
		Ok(())
	}
}

/// Display for Sourcedata.
///
/// All Sourcedata can be written in a form such that it can be read again.
impl fmt::Display for Sourcedata {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use data_structures::{Coredata::*, Function, Macro};
		// What needs to be printed
		enum Queue<'a> {
			Close,
			Data(&'a Sourcedata, Context),
		}
		#[derive(Clone, Copy)]
		enum Context {
			Cell, // The nested cell in the top level
			Runnable, // The nested function of a function or macro
			Run, // The top level of a function or macro
			TopLevel, // The base level used for printing arbitrary things
		}
		let mut queue: Vec<Queue> = Vec::with_capacity(VEC_CAPACITY);
		// Current printing context, if it's a function or macro we do not write (symbol X)
		// Is it ONLY used for symbols? It should be... let me think...
		// Yes. Let's try it. We just store false/true
		let mut spacer = false;
		queue.push(Queue::Data(self, Context::TopLevel));
		while let Some(elem) = queue.pop() {
			macro_rules! spacify { () => { if spacer { write![f, " "]?; } }; }
			match elem {
				Queue::Close => {
					write![f, ")"]?;
				}
				Queue::Data(ref data, ref context) => {
					match data.1 {
						Boolean(state) => {
							spacify![];
							write![f, "{}", state]?;
							spacer = true;
						}
						Cell(ref head, ref tail) => {
							spacify![];
							match *context {
								Context::Run => {
									write![f, "("]?;
									queue.push(Queue::Close);
								}
								Context::Cell => {}
								Context::Runnable => {}
								Context::TopLevel => {
									write![f, "(list "]?;
									queue.push(Queue::Close);
								}
							}
							queue.push(Queue::Data(tail,
								if let Context::Run = *context { Context::Runnable }
								else if let Context::Runnable = *context { Context::Runnable }
								else { Context::Cell }));
							queue.push(Queue::Data(head,
								if let Context::Run = *context { Context::Run }
								else if let Context::Runnable = *context { Context::Run }
								else { Context::TopLevel }));
							spacer = false;
						}
						Error(ref arg) => {
							spacify![];
							write![f, "(error"]?;
							if let Coredata::Null() = arg.1 {
								write![f, ")"]?;
							} else {
								queue.push(Queue::Close);
								queue.push(Queue::Data(arg, Context::TopLevel));
							}
							spacer = true;
						}
						Function(Function::Builtin(.., ref name)) => {
							spacify![];
							write![f, "{}", name]?;
							spacer = true;
						}
						Function(Function::Library(ref params, ref code)) => {
							spacify![];
							// HEADER and PARAMETER LIST
							write![f, "(function ("]?;
							let mut first = true;
							for i in params.iter() {
									if ! first {
										write![f, " "]?;
									}
									write![f, "{}", Into::<&str>::into(i)]?;
									first = false;
							}
							write![f, ")"]?;
							// QUEUE code
							queue.push(Queue::Close);
							for i in code.iter() {
								queue.push(Queue::Data(i, Context::Run));
							}
							spacer = true;
						}
						Integer(ref arg) => {
							spacify![];
							write![f, "{}", arg]?;
							spacer = true;
						}
						Macro(Macro::Builtin(.., ref name)) => {
							spacify![];
							write![f, "{}", name]?;
							spacer = true;
						}
						Macro(Macro::Library(ref param, ref code)) => {
							spacify![];
							write![f, "(macro {}", Into::<&str>::into(param)]?;
							// QUEUE code
							queue.push(Queue::Close);
							for i in code.iter() {
								queue.push(Queue::Data(i, Context::Run));
							}
							spacer = true;
						}
						Null() => {
							if let Context::Cell = context {
							} else if let Context::Runnable = context {
								// Do nothing
							} else {
								spacify![];
								write![f, "()"]?;
							}
							spacer = true;
						}
						String(ref arg) => {
							spacify![];
							macro_rules! is_plainly_printable {
								($i:ident) => {
									// TODO remove () around cast: rustc panics because it thinks it's a generic
									!$i.is_whitespace() && $i != '(' && $i != ')' && $i as u32 > 0x1F &&
									(($i as u32) < 0x7F || $i as u32 > 0x9F)
								};
							}
							write![f, "(\""]?;
							if !arg.is_empty() { write![f, " "]?; }
							let mut prev_char = ' ';
							let mut rle = 0;
							let rle_write = |f: &mut fmt::Formatter, prev_char: char, rle: usize| -> Result<(), fmt::Error> {
								if rle > 0 {
									if rle == 1 {
										write![f, "({})", prev_char as u32]?;
									} else {
										write![f, "({} {})", prev_char as u32, rle]?;
									}
								}
								Ok(())
							};
							for (n, ch) in arg.chars().enumerate() {
								if is_plainly_printable![ch] {
									if rle > 0 {
										if prev_char == ' ' && rle == 1 && n > 1 {
											write![f, " "]?;
										} else {
											rle_write(f, prev_char, rle)?;
										}
									}
									write![f, "{}", ch]?;
									rle = 0;
								} else if ch != prev_char && rle > 0 {
									rle_write(f, prev_char, rle)?;
									rle = 1;
								} else {
									rle += 1;
								}
								prev_char = ch;
							}
							rle_write(f, prev_char, rle)?;
							write![f, ")"]?;
							spacer = true;
						}
						Symbol(ref symbol) => {
							spacify![];
							if let Context::Runnable = *context {
								write![f, "{}", Into::<&str>::into(symbol)]?;
							} else if let Context::Run = *context {
								write![f, "{}", Into::<&str>::into(symbol)]?;
							} else {
								write![f, "(@ {})", Into::<&str>::into(symbol)]?;
							}
							spacer = true;
						}
						_ => {}
					}
				}
			}
		}
		Ok(())
	}
}

impl Sourcedata {
	/// Return the head of a cell, unwind if not a cell.
	pub fn head(&self) -> Option<Arc<Sourcedata>> {
		if let Sourcedata(_, Coredata::Cell(ref head, _)) = *self {
			Some(head.clone())
		} else {
			None
		}
	}
	/// Return the tail of a cell, unwind if not a cell.
	pub fn tail(&self) -> Option<Arc<Sourcedata>> {
		if let Sourcedata(_, Coredata::Cell(_, ref tail)) = *self {
			Some(tail.clone())
		} else {
			None
		}
	}
	/// Compute the size of the object in element count.
	pub fn len(&self) -> Option<usize> {
		if let Coredata::String(ref string) = self.1 {
			return Some(string.len());
		}
		let mut current = self;
		let mut length = 0;
		loop {
			match current.1 {
				Coredata::Cell(_, ref tail) => {
					length += 1;
					current = &*tail;
				}
				Coredata::Null() => {
					return Some(length);
				}
				_ => {
					return None;
				}
			}
		}
	}
}

impl Default for Source {
	fn default() -> Source {
		Source {
			line: 1,
			column: 1,
			source: "unknown".into(),
		}
	}
}

impl fmt::Display for Source {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write![f, "{}:{}:{}", self.line, self.column, self.source]
	}
}

impl<'a> convert::From<&'a Source> for Arc<Sourcedata> {
	fn from(src: &'a Source) -> Arc<Sourcedata> {
		use data_structures::Coredata::*;
		rcs(Cell(
			rcs(Integer(src.line.into())),
			rcs(Cell(
				rcs(Integer(src.column.into())),
				rcs(Cell(rcs(String(src.source.clone())), rcs(Null()))),
			)),
		))
	}
}

impl Default for ParseState {
	fn default() -> ParseState {
		ParseState {
			current_read_position: Source::default(),
			start_of_current_lexeme: Source::default(),
			unmatched_opening_parentheses: Vec::with_capacity(VEC_CAPACITY),
			token: String::from(""),
			stack: vec![vec![]],
			error: None,
		}
	}
}

impl ParseState {
	pub fn from(source: &str) -> ParseState {
		let mut state = ParseState::default();
		state.current_read_position = Source {
			line: 1,
			column: 1,
			source: source.into(),
		};
		state
	}
}

// //////////////////////////////////////////////////////////
// Utility functions
// //////////////////////////////////////////////////////////

pub fn arity_mismatch(expected_min: usize, expected_max: usize, got: usize) -> String {
	if expected_min == expected_max {
		format!["arity mismatch: expected {} but got {}", expected_min, got]
	} else if expected_min < expected_max && expected_min == 0 {
		format![
			"arity mismatch: expected <={} but got {}",
			expected_max,
			got,
		]
	} else if expected_min < expected_max && expected_max == usize::MAX {
		format![
			"arity mismatch: expected >={} but got {}",
			expected_min,
			got,
		]
	} else {
		format![
			"arity mismatch: expected >={} and <={} but got {}",
			expected_min,
			expected_max,
			got,
		]
	}
}

pub fn not_found(string: &str) -> String {
	format!["variable not found: {}", string]
}

/// Maps a linked list of data into a vector of data.
pub fn collect_cell_into_revvec(data: &Arc<Sourcedata>) -> Vec<Arc<Sourcedata>> {
	let mut to_return = vec![];
	let mut current = data.clone();
	loop {
		current = if let Sourcedata(_, Coredata::Cell(ref head, ref tail)) = *current {
			to_return.push(head.clone());
			tail.clone()
		} else {
			break;
		}
	}
	to_return.reverse();
	to_return
}

/// Maps a linked list of symbols into a vector of strings.
pub fn collect_cell_of_symbols_into_vec(data: &Arc<Sourcedata>) -> Option<Vec<Symbol>> {
	let mut ret = vec![];
	let mut current = data.clone();
	if let Coredata::Cell(..) = current.1 {
		// Ok
	} else if let Coredata::Null() = current.1 {
		// Ok
	} else {
		return None;
	}
	loop {
		current = if let Sourcedata(_, Coredata::Cell(ref head, ref tail)) = *current {
			if let Coredata::Symbol(ref symbol) = head.1 {
				ret.push(symbol.clone());
				tail.clone()
			} else {
				return None;
			}
		} else {
			break;
		}
	}
	Some(ret)
}
/* /// Maps a linked list of symbols into a vector of strings. */
/* pub fn collect_cell_of_symbols_into_vec_string(data: &Rc<Sourcedata>) -> Option<Vec<String>> { */
/* 	let mut ret = vec![]; */
/* 	let mut current = data.clone(); */
/* 	if let Coredata::Cell(..) = current.1 { */
/* 		// Ok */
/* 	} else if let Coredata::Null() = current.1 { */
/* 		// Ok */
/* 	} else { */
/* 		return None; */
/* 	} */
/* 	loop { */
/* 		current = if let Sourcedata(_, Coredata::Cell(ref head, ref tail)) = *current { */
/* 			if let Coredata::Symbol(ref string) = head.1 { */
/* 				ret.push(string.into()); */
/* 				tail.clone() */
/* 			} else { */
/* 				return None; */
/* 			} */
/* 		} else { */
/* 			break; */
/* 		} */
/* 	} */
/* 	Some(ret) */
/* } */

/// Takes the intersection of two sets.
pub fn compute_intersection<'a>(a: &'a [String], b: &'a [String]) -> Vec<&'a String> {
	let mut intersection: Vec<&'a String> = Vec::with_capacity(VEC_CAPACITY);
	for i in a {
		if b.contains(i) {
			intersection.push(i);
		}
	}
	intersection
}

/// Takes the union of two sets.
pub fn compute_union(a: &[String], b: &[String]) -> Vec<String> {
	let mut c = a.to_vec();
	for i in b {
		if !a.contains(i) {
			c.push(i.clone());
		}
	}
	c
}

/// Get the name associated with the data type.
pub fn data_name(data: &Sourcedata) -> String {
	match data.1 {
		Coredata::Boolean(..) => "Boolean",
		Coredata::Cell(..) => "Cell",
		Coredata::Error(..) => "Error",
		Coredata::Function(Function::Builtin(..)) => "Builtin Function",
		Coredata::Function(Function::Library(..)) => "Function",
		Coredata::Integer(..) => "Integer",
		Coredata::Internal(..) => "Internal",
		Coredata::Macro(..) => "Macro",
		Coredata::Null(..) => "Null",
		Coredata::String(..) => "String",
		Coredata::Symbol(..) => "Symbol",
		Coredata::Table(..) => "Table",
	}.into()
}

/// Unwind and trace with an error message if it is Some.
///
/// Mixes unwind and tracing from an error's invocation. Any time an unwind
/// happens `env.result` will contain an error with a string containing the stack
/// trace an addition to the error provided.
pub fn err(
	source: &Option<Source>,
	error: &Option<(Option<Source>, String)>,
	program: &mut Program,
	env: &mut Env,
) {
	let error = if let Some((ref src, ref error)) = *error {
		let mut temp = vec![];
		if src.is_none() {
			temp.push(rc(
				Sourcedata(source.clone(), Coredata::String(error.clone())),
			));
		} else {
			if source != src {
				temp.push(rc(Sourcedata(
					source.clone(),
					Coredata::String("called from here".into()),
				)));
			}
			temp.push(rc(Sourcedata(src.clone(), Coredata::String(error.clone()))));
		}
		let trace = internal_trace(&mut temp, env);
		Some(trace)
	} else {
		None
	};
	if let Some(error) = error {
		env.params.push(vec![rcs(Coredata::Error(error))]);
		unwind(program, env);
		if env.params.pop().is_none() {
			panic!["Stack corruption"];
		}
	}
}

/// Create a string of the entire program stack.
pub fn internal_trace(program: &mut Program, _: &mut Env) -> Arc<Sourcedata> {
	use data_structures::Coredata::*;
	let null = rcs(Coredata::Null());
	let mut lst = null.clone();
	for i in program.iter().rev() {
		if let Sourcedata(Some(ref source), ..) = **i {
			lst = rcs(Cell(
				rcs(Cell(source.into(), rcs(Cell(i.clone(), rcs(Null()))))),
				lst.clone(),
			));
		} else {
			lst = rcs(Cell(
				rcs(Cell(rcs(Null()), rcs(Cell(i.clone(), rcs(Null()))))),
				lst.clone(),
			));
		}
	}
	lst
}

/// Optimizes tail calls by seeing if the current `params` can be merged with the top of the stack.
///
/// If the top of the stack contains `Commands::Deparize`, then the variables to be popped
/// are merged into that [top] object. This is all that's needed to optimize tail calls.
pub fn optimize_tail_call(program: &mut Program, env: &mut Env, params2: &[Symbol]) -> Deparize {
	if let Some(top) = program.pop() {
		match top.1 {
			Coredata::Internal(Commands::Deparize(ref content2)) => {
				let mut content = content2.clone();
				for i in params2 {
					if content.check_preexistence_and_merge_single(i) {
						if env.pop(i).is_some() {
							// OK
						} else {
							panic!["Store inconsistency; entry empty"];
						}
					}
				}
				// TODO remove this clone, quite unnecessary
				content.clone()
			}
			_ => {
				let mut deparize = Deparize::default();
				program.push(top.clone()); // Put top back on the program stack
				for i in params2 {
					deparize.check_preexistence_and_merge_single(i);
				}
				deparize
			}
		}
	} else {
			let mut deparize = Deparize::default();
			for i in params2 {
				deparize.check_preexistence_and_merge_single(i);
			}
			deparize
	}
}

pub fn optional_source(source: &Option<Source>) -> String {
	if let Some(ref source) = *source {
		format!["{}", source]
	} else {
		String::from("_")
	}
}


// TODO change from panic to unwind, but can we be safe about such a serious error by
// unwinding? Maybe a stop function that freezes the interpreter...
/// Pops the specified parameters from the stack.
///
/// If the parameters do not exist then there's an internal programmer error and
/// this function will panic.
pub fn pop_parameters(_: &mut Program, env: &mut Env, args: &Deparize) {
	for arg in args.iter() {
		if env.pop(arg).is_some() {
			// OK
		} else {
			panic!["Store entry does not exist"];
		}
	}
}

/// Alias for `Rc::new(_)`.
pub fn rc<T>(rc: T) -> Arc<T> {
	Arc::new(rc)
}

/// Alias for `Rc::new(Sourcedata(None, _))`.
pub fn rcs(rcs: Coredata) -> Arc<Sourcedata> {
	rc(Sourcedata(None, rcs))
}

pub fn find_earliest_depar(program: &mut Program) -> Option<&mut Deparize> {
	for i in program.iter_mut().rev() {
		if let Some(&mut Sourcedata(_, Coredata::Internal(Commands::Deparize(ref mut dep)))) = Arc::get_mut(i) {
			return Some(dep);
		}
	}
	None
}

/// Unwinds the stack until first wind is encountered.
///
/// Preserves stack consistency (pops parameters when necessary).
pub fn unwind(program: &mut Program, env: &mut Env) -> Option<(Option<Source>, String)> {
	let result;
	if let Some(param) = env.params.last() {
		if let Some(last) = param.last() {
			result = last.clone();
		} else {
			result = rcs(Coredata::Null());
		}
	} else {
		result = rcs(Coredata::Null());
	}
	env.set_result(result);
	while let Some(top) = program.pop() {
		match top.1 {
			Coredata::Internal(Commands::Deparize(ref arguments)) => {
				pop_parameters(program, env, arguments);
			}
			Coredata::Internal(Commands::Call(..)) => {
				env.params.pop();
			}
			Coredata::Internal(Commands::Wind) => {
				break;
			}
			_ => {}
		}
	}
	None
}

#[cfg(test)]
mod tests {
	fn test_string(input: &str, output: &str) {
		use data_structures::{Coredata, Sourcedata};
		assert_eq![output, format!["{}", Sourcedata(None, Coredata::String(input.to_string()))]];
	}
	#[test]
	fn string_writing() {
		test_string("", "(\")");
		test_string(" ", "(\" (32))");
		test_string(" X", "(\" (32)X)");
		test_string("X Y", "(\" X Y)");
		test_string(" X Y", "(\" (32)X Y)");
		test_string(" X Y\n", "(\" (32)X Y(10))");
		test_string("Lorem ipsum dolor sit amet", "(\" Lorem ipsum dolor sit amet)");
		test_string("  \n  ", "(\" (32 2)(10)(32 2))");
		test_string("  \n", "(\" (32 2)(10))");
		test_string(" \n", "(\" (32)(10))");
		test_string(" \n\t", "(\" (32)(10)(9))");
		test_string(" \n\n\t", "(\" (32)(10 2)(9))");
		test_string("A\n\n\t", "(\" A(10 2)(9))");
		test_string("A\n\nBC\t", "(\" A(10 2)BC(9))");
		test_string("A\nD\nBC\t", "(\" A(10)D(10)BC(9))");
	}
}
