//! [![Build Status](https://travis-ci.com/kurtlawrence/cmdtree.svg?branch=master)](https://travis-ci.com/kurtlawrence/cmdtree)
//! [![Latest Version](https://img.shields.io/crates/v/cmdtree.svg)](https://crates.io/crates/cmdtree) 
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/cmdtree) 
//! [![codecov](https://codecov.io/gh/kurtlawrence/cmdtree/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/cmdtree)
//! 
//! (Rust) commands tree.
//! 
//! See the [rs docs](https://docs.rs/cmdtree/).
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/cmdtree)
//! 
//! Currently WIP placeholder.

#![warn(missing_docs)]

use colored::*;
use linefeed::{Interface, ReadResult};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

pub mod builder;
mod parse;

use self::parse::LineResult;
pub use builder::{Builder, BuilderChain};

/// A constructed command tree.
///
/// Most of the time a user will want to use `run()` which will handle all the parsing and navigating of the tree.
/// Alternatively, `parse_line` can be used to simulate a read input and update the command tree position.
///
/// To construct a command tree, look at the [`builder` module](./builder).
pub struct Commander<'r> {
	root: Rc<SubClass<'r>>,
	current: Rc<SubClass<'r>>,
	path: String,
}

impl<'r> Commander<'r> {
	/// Return the path of the current class, separated by `.`.
	///
	/// # Example
	/// ```rust
	/// use cmdtree::*;
	/// let mut cmder = Builder::default_config("base")
	///		.begin_class("one", "")
	///		.begin_class("two", "")
	///		.into_commander().unwrap();
	///
	///	assert_eq!(cmder.path(), "base");
	///	cmder.parse_line("one two", true,  &mut std::io::sink());
	///	assert_eq!(cmder.path(), "base.one.two");
	/// ```
	pub fn path(&self) -> &str {
		&self.path
	}

	/// Run the `Commander` interactively.
	/// Consumes the instance, and blocks the thread until the loop is exited.
	/// Reads from `stdin` using [`linefeed::Interface`](https://docs.rs/linefeed/0.5.4/linefeed/interface/struct.Interface.html).
	///
	/// This is the most simple way of using a `Commander`.
	pub fn run(mut self) {
		let interface = Interface::new("commander").expect("failed to start interface");
		let mut exit = false;

		while !exit {
			interface
				.set_prompt(&format!("{}=> ", self.path().bright_cyan()))
				.expect("failed to set prompt");

			match interface.read_line() {
				Ok(ReadResult::Input(s)) => match self.parse_line(&s, true, &mut std::io::stdout())
				{
					LineResult::Exit => exit = true,
					_ => (),
				},
				_ => (),
			}
		}
	}
}

#[derive(Debug, PartialEq)]
struct SubClass<'a> {
	name: String,
	help: &'a str,
	classes: Vec<Rc<SubClass<'a>>>,
	actions: Vec<Action<'a>>,
}

impl<'a> SubClass<'a> {
	fn with_name(name: &str, help_msg: &'a str) -> Self {
		SubClass {
			name: name.to_lowercase(),
			help: help_msg,
			classes: Vec::new(),
			actions: Vec::new(),
		}
	}
}

struct Action<'a> {
	name: String,
	help: &'a str,
	closure: RefCell<Box<FnMut(&[&str]) + 'a>>,
}

impl<'a> Action<'a> {
	fn call(&self, arguments: &[&str]) {
		let c = &mut *self.closure.borrow_mut();
		c(arguments);
	}

	#[cfg(test)]
	fn blank_fn(name: &str, help_msg: &'a str) -> Self {
		Action {
			name: name.to_lowercase(),
			help: help_msg,
			closure: RefCell::new(Box::new(|_| ())),
		}
	}
}

impl<'a> PartialEq for Action<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.help == other.help
	}
}

impl<'a> fmt::Debug for Action<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Action {{ name: {}, help: {} }}", self.name, self.help)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn subclass_with_name_test() {
		let sc = SubClass::with_name("NAME", "Help Message");
		assert_eq!(&sc.name, "name");
		assert_eq!(sc.help, "Help Message");
	}

	#[test]
	fn action_debug_test() {
		let a = Action::blank_fn("action-name", "help me!");
		assert_eq!(
			&format!("{:?}", a),
			"Action { name: action-name, help: help me! }"
		);
	}

	#[test]
	fn current_path_test() {
		let mut cmder = Builder::default_config("base")
			.begin_class("one", "")
			.begin_class("two", "")
			.into_commander()
			.unwrap();

		let w = &mut std::io::sink();

		assert_eq!(cmder.path(), "base");

		cmder.parse_line("one two", true, w);
		assert_eq!(cmder.path(), "base.one.two");

		cmder.parse_line("c", true, w);
		assert_eq!(cmder.path(), "base");

		cmder.parse_line("one", true, w);
		assert_eq!(cmder.path(), "base.one");
	}
}
