# Overview

The TOML file describes a **contract**, which is a logical specification of constraints (preconditions and postconditions), as well as the inputs and environment required for testing a particular Rust function. It can also include references to external dependencies and `use` statements.

## Top-Level Structure

The TOML representation of a `Contract` generally looks like this:

```toml
description = "Description of what the contract is testing."

# Precondition and postcondition are Rust expressions given as strings.
precondition = "/* some Rust expression */"
postcondition = "/* some Rust expression */"

# `function_tested` is a Rust path (like `my_crate::my_module::my_function`).
function_tested = "my_crate::my_function"

# `use_statements` are arrays of Rust `use` items, like `use std::collections::HashMap;`.
use_statements = [
    "use std::collections::HashMap;",
    "use serde::Deserialize;",
]

# Inputs are arrays of named inputs, each specifying either a type or value kind.
[[inputs]]
name = "input_data"
# This input is of kind "Value" with a type and possibly aliases.
typ = "Vec<u8>"
aliases = ["data", "bytes"]

[[inputs]]
name = "T"
# This input is of kind "Type", providing type bounds.
bounds = "where T: Clone + Default"

# Dependencies can be specified as a map of package names to dependency specs.
[dependencies]
serde = { version = "1.0", features = ["derive"] }
my_crate = { path = "../my_crate" }

```

Testify reads a list of contracts in a toml file.

# Fields in Detail

### `Contract`

**Table keys:**

- **`description`** *(string)*  
  A human-readable description of the contract. This is used for documentation or explaining what the contract tests or enforces.

- **`precondition`** *(string representing a Rust expression)*  
  A Rust expression defining the precondition. Before running or testing the function, the precondition should hold true. If no precondition is provided, it defaults to `true`.

  For example:  
  ```toml
  precondition = "x > 0"
  ```

- **`postcondition`** *(string representing a Rust expression)*  
  A Rust expression defining the postcondition. After running the function, this condition must hold. If not specified, it defaults to `true`.

  For example:  
  ```toml
  postcondition = "result.is_ok()"
  ```

- **`span`** *(internal detail)*  
  Represents a code location. Automatically defaults to a dummy span. Usually not specified by the user.  
  ```toml
  # This is usually omitted or left as default
  ```

- **`dependencies`** *(map of strings to dependency specs)*  
  A TOML table where keys are dependency names and values are arbitrary TOML definitions specifying those dependencies. This mirrors Cargo dependency specification.

  Example:
  ```toml
  [dependencies]
  serde = { version = "1.0", features = ["derive"] }
  regex = "1.5"
  my_local_crate = { path = "../my_local_crate" }
  ```

- **`use_statements`** *(array of strings)*  
  An array of Rust `use` statements. Each string should be a valid Rust `use` item.  
  For example:
  ```toml
  use_statements = [
      "use std::collections::HashMap;",
      "use crate::utils::*;"
  ]
  ```

- **`function_tested`** *(string representing a Rust path)*  
  A path to the Rust function that this contract tests. If omitted, it might mean the contract does not target a specific function. Only contracts with a non-empty `function_tested` field are considered for coverage or LLM generation.

  For example:
  ```toml
  function_tested = "crate::my_module::target_function"
  ```

- **`inputs`** *(array of `Input` tables)*  
  Each entry describes one input necessary for the contract. Inputs can either define a value-like input or a type parameter. See **`Input`** below.

### `Input`

Each `Input` is specified within `[[inputs]]` arrays. An `Input` always has:

- **`name`** *(string)*  
  The name of the input. If it is a value input, this corresponds to a variable name. If it is a type input, this corresponds to a generic type parameter.

- **`kind`** *(implicit, derived from provided fields)*  
  Determines whether the input is a value input or a type input based on which fields are provided.

Two kinds of inputs are supported:

1. **Value Input**
   
   A value input includes:
   - **`typ`** *(string representing a Rust type)*  
     The Rust type of the input. This must be a valid Rust type.
   
   - **`aliases`** *(array of strings, optional)*  
     Additional names that can refer to the same input. This is useful to introduce multiple times the same value without having to clone it.

   Example:
   ```toml
   [[inputs]]
   name = "input_data"
   typ = "Vec<u8>"
   aliases = ["data", "bytes"]
   ```

2. **Type Input**
   
   A type input includes:
   - **`bounds`** *(string representing a Rust where-clause)*  
     A Rust `where` clause specifying trait bounds on a generic type parameter.

   Example:
   ```toml
   [[inputs]]
   name = "T"
   bounds = "where T: Clone + Default"
   ```

---

### `InputKind`

The `kind` of an input is inferred based on the provided fields:

- **`Value`** kind if `typ` is given.
- **`Type`** kind if `bounds` is given.

You will typically not specify `kind` explicitly; instead you provide either `typ` (for value inputs) or `bounds` (for type inputs).

# Example of a contract in TOML

```toml
description = "Check that `my_function` returns a positive number for positive inputs."

precondition = "x > 0"
postcondition = "result > 0"
function_tested = "crate::my_function"

use_statements = [
    "use std::collections::HashMap;",
    "use crate::helpers::some_helper;",
]

[[inputs]]
name = "x"
typ = "i32"
aliases = ["input_x"]

[[inputs]]
name = "T"
bounds = "where T: Default + Clone"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
```
