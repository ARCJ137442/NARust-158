# NARust 158

**Simplified Chinese** | [English](README.en.md)

🕒Last Updated: 2024-08-19

## Introduction

<!-- <image src="./docs/image/opennars-logo-modified-with-rust.svg" width=200 style="display: block; margin: auto;"></image> -->

A [Non-Axiomatic Reasoning System](http://www.opennars.org/) implemented in [Rust](https://www.rust-lang.org/), based on [OpenNARS 1.5.8](https://github.com/patham9/opennars_declarative_core).

## Online Demo

Thanks to the integration of Rust and WebAssembly, this reasoning system has a [web version](https://arcj137442.github.io/demo-158-dev/) and can be run directly in browsers that support WebAssembly.

(Browsers that support WebAssembly: Chrome 57+, EDGE 16+, Safari 11+, Firefox 52+, Opera 44+, ...)

If you want to compile it to the web by yourself, you can refer to the [RustWasm](https://rustwasm.github.io/) and the [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) tool, and you can search for related resources by yourself.

## Quick Start

### Prerequisites

1. Install the [**Rust**](https://www.rust-lang.org/tools/install) compiler toolchain in the system
2. Ensure that the `cargo` command is available after installation

![Expected result of entering `cargo` in the system command line after installing Cargo](./docs/image/installed-cargo.png)

### Try It: Run Immediately

In an **online** environment, run the following command directly:

```bash
cargo install narust-158
```

As of today (2024-08-19), this command will install the following two binaries in the system:

- `narust_158_shell`: Convenient for user interaction, allowing direct input of Narsese statements and values (specify the reasoning cycle step period)
- `narust_158_batch`: Convenient for external integration, unified input of NAVM commands, and input of fixed single-line JSON text format

### Advanced: Source Code Compilation

#### Get the Source Code

You can get the source code directly from the project repository on GitHub:

```bash
git clone https://github.com/arcj137442/narust-158.git
```

You should expect the following reaction: Git fetches the source code from the GitHub repository and downloads it into a specific directory called `narust-158`.

After the project is published to **crates.io**, you can get it in the Rust project directory with the following command:

```bash
cargo add narust-158
```

#### Local Compilation

The source code obtained from GitHub can be built by entering the following command in the command line at the root directory of `narust-158`:

```bash
cargo build
```

You should expect Cargo to automatically download compilation dependencies and ultimately complete the compilation of the project binary files:

```bash
[...]> cargo build
   Compiling narust-158 vX.X.X ([...])
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

#### Build and Run

At this point, you can use the command `cargo run` to run the built binary files:

```bash
cargo run --bin narust_158_shell
```

or

```bash
cargo run --bin narust_158_batch
```

Expect: The command line cursor opens a new line and waits for user input.

After entering the following NAVM command,

```navm-cmd
nse <A --> B>.
nse <B --> C>.
nse <A --> C>?
cyc 20
```

Expect the following output:

```plaintext
nse <A --> B>.
[IN] $0.8000;0.8000;0.9500$ <A --> B>. %1.0000;0.9000%
nse <B --> C>.
[IN] $0.8000;0.8000;0.9500$ <B --> C>. %1.0000;0.9000%
nse <A --> C>?
[IN] $0.9000;0.9000;1.0000$ <A --> C>?
cyc 20
[ANSWER] <A --> C>. %1.0000;0.8100%
```

(↑In `shell`)

```plaintext
nse <A --> B>.
{"type":"IN","content":"In: $0.80;0.80;0.95$ (A --> B). %1.00;0.90%","narsese":"$0.8000;0.8000;0.9500$ <A --> B>. %1.0000;0.9000%"}
nse <B --> C>.
{"type":"IN","content":"In: $0.80;0.80;0.95$ (B --> C). %1.00;0.90%","narsese":"$0.8000;0.8000;0.9500$ <B --> C>. %1.0000;0.9000%"}
nse <A --> C>?
{"type":"IN","content":"In: $0.90;0.90;1.00$ (A --> C)?","narsese":"$0.9000;0.9000;1.0000$ <A --> C>?"}
cyc 20
{"type":"ANSWER","content":"Answer: (A --> C). %1.0000;0.8100%{16 : 2;1}","narsese":"<A --> C>. %1.0000;0.8100%"}
```

(↑In `batch`)

## Project Overview

### Declaration

Regarding the meaning of the term 'OpenNARS': Unless specifically distinguished, it refers by default to [`OpenNARS 1.5.8`](https://github.com/patham9/opennars_declarative_core), and also refer to the individual's [Chinese Notes Appendix Custom Edition](https://github.com/ARCJ137442/OpenNARS-158-dev).

- 📌The project structure is mainly based on the 'Chinese Notes Appendix Custom Edition' (hereinafter referred to as 'Revised OpenNARS').
- ⚠️Comments and notes in the project are all written in Chinese.

### System Module Architecture

The main directory structure of the entire system is as follows:

```plaintext
narust-158
├── docs
├── src
│   ├── bin:        Executable compilation entry
│   ├── control:    Control mechanism
│   ├── entity:     Entity structure
│   ├── inference:  Inference mechanism
│   ├── language:   Knowledge representation language
│   ├── util:       Internal utility functions
│   ├── vm:         Virtual machine interface and built-in implementation
│   ├── global.rs:  Global parameters
│   ├── lib.rs:     Library compilation entry
│   ├── symbols.rs: Global symbol constants, corresponding to OpenNARS `nars.io.Symbols`
│   └── ...
├── Cargo.toml
└── ...
```

#### Knowledge Representation Language

Language module `src/language`: Definitions and processing logic related to 'terms'

```plaintext
language
├── term_impl: Concrete implementation of term structure
│   ├── base: Basic functionality
│   │   ├── construct.rs:  Constructors
│   │   ├── conversion.rs: Type conversion
│   │   ├── property.rs:   Properties
│   │   ├── serde.rs:      Serialization and deserialization
│   │   ├── structs.rs:    Struct definitions
│   │   └── ...
│   ├── dialect: Dialect syntax
│   │   ├── mod.rs:              Dialect parser
│   │   └── narust_dialect.pest: Syntax definition
│   ├── features: Corresponding to OpenNARS features
│   │   ├── compound_term.rs: Corresponding to OpenNARS class `CompoundTerm`
│   │   ├── image.rs:         Corresponding to OpenNARS class `Image`
│   │   ├── statement.rs:     Corresponding to OpenNARS class `Statement`
│   │   ├── term.rs:          Corresponding to OpenNARS class `Term`
│   │   ├── variable.rs:      Corresponding to OpenNARS class `Variable`
│   │   └── ...
│   ├── term_making.rs:      Corresponding to OpenNARS `MakeTerm.java`
│   ├── variable_process.rs: Corresponding to OpenNARS `VariableProcess.java`
│   └── ...
└── ...
```

Entity module `src/entity`: Definitions of structures such as 'truth value', 'budget value', 'statement', 'term chain & task chain', 'concept', 'timestamp', 'task', etc.

```plaintext
entity
├── float_values: 'Floating point values' used by both the language mechanism and the control mechanism
│   ├── budget_value.rs: Budget value
│   ├── truth_value.rs:  Truth value
│   ├── short_float.rs:  Short floating point (four decimal places)
│   └── ...
├── sentence: Language mechanism definitions related to 'statements'
│   ├── impls:             Initial implementation (statements, judgments, questions)
│   ├── judgement.rs:     Unified 'judgment' interface
│   ├── punctuation.rs:   Based on enumeration 'punctuation' definition
│   ├── question.rs:      Unified 'question' interface
│   ├── sentence_trait.rs: Unified 'sentence' interface
│   └── ...
├── linkages: Control mechanism definitions related to 'linkages'
│   ├── t_link.rs:             Unified 'link' interface
│   ├── t_linkage.rs:          General 'linkage' structure
│   ├── task_link.rs:          Task chain
│   ├── term_link_template.rs: Term link template
│   ├── term_link.rs:          Term link
│   └── ...
├── concept.rs: Control mechanism structure 'concept'
├── item.rs:    Control mechanism interface 'item'
├── stamp.rs:   Language mechanism structure 'timestamp'
├── task.rs:    Control mechanism structure 'task'
└── ...
```

(This structure originates from OpenNARS 1.5.8, and the code has not been completely separated in terms of inference and control mechanisms)

#### Inference Control Mechanism

NARS Storage Container `src/storage`: Storage container related to "entities"

```plaintext
storage
├── bag: Basic container "bag" based on the control mechanism of "pseudo-random priority queue"
│   ├── distributor.rs: Pseudo-random distributor based on priority with a triangular distribution
│   ├── impl_tables.rs: Auxiliary "name table" and "level table" structures
│   ├── impl_v1.rs: The final exported "first generation implementation"
│   └── ...
├── buffer.rs: "Buffer" structure used in "concepts"
├── memory.rs: Overall container "memory area" for storing "concepts"
├── rank_table.rs: "Ranking table" structure used in "concepts"
└── ...
```

NARS Inference Function `src/inference`: The process of handling various "entities" in the "container" based on NAL and "knowledge representation language" mechanism

```plaintext
inference
├── engine: Universal "inference engine" interface
│   ├── inference_engine.rs: Definition and interface of the inference engine
│   └── ...
├── functions: Truth functions, budget functions, etc.
│   ├── budget_functions.rs: Code related to "budget functions" in NAL, corresponding to `nars.inference.BudgetFunctions`
│   ├── truth_functions.rs: "Truth functions" in NAL, corresponding to `nars.inference.TruthFunctions`
│   ├── utility_functions.rs: Code related to "extended logical operations" in NAL, corresponding to `nars.inference.UtilityFunctions`
│   └── ...
├── rules: Specific NAL inference rules
│   ├── table: Rule dispatch table
│   │   ├── entry.rs: Rule dispatch entry, corresponding to `nars.inference.RuleTables`
│   │   ├── syllogistic.rs: Dispatch related to "syllogistic rules"
│   │   ├── compositional.rs: Dispatch related to "compositional rules"
│   │   └── ...
│   ├── compositional_rules.rs: Compositional rules, corresponding to `nars.inference.CompositionalRules`
│   ├── local_rules.rs: Local rules, corresponding to `nars.inference.LocalRules`
│   ├── matching_rules.rs: Matching rules, corresponding to `nars.inference.MatchingRules`
│   ├── structural_rules.rs: Structural rules, corresponding to `nars.inference.StructuralRules`
│   ├── syllogistic_rules.rs: Syllogistic rules, corresponding to `nars.inference.SyllogisticRules`
│   ├── transform_rules.rs: Transformation rules, corresponding to `nars.inference.TransformRules`
│   └── ...
├── traits
│   ├── budget.rs: Abstract interface related to "budget", shared by "budget value", "task", "concept", etc.
│   ├── evidential.rs: Abstract interface related to "evidence base", shared by "timestamp", "statement", "task", etc.
│   ├── truth.rs: Abstract interface related to "truth", shared by "truth value", "judgment", etc.
│   └── ...
├── budget_inference.rs: "Budget inference" involving "link feedback"
├── local_inference.rs: "Direct inference" involving "belief revision" and "question answering"
└── ...
```

NARS Control Mechanism `src/control`: Functions closely related to the "reasoner" on top of "containers" and "inference rules"

```plaintext
control
├── context: "Inference context" function in the control mechanism
│   ├── context_concept.rs: Concept inference context
│   ├── context_direct.rs: Direct inference context
│   ├── context_transform.rs: Transformation inference context
│   ├── derivation.rs: Related functions of inference derivation
│   ├── reason_context.rs: Unified "inference context" interface
│   └── ...
├── process: Runtime functions related to "work cycle"
│   ├── concept_linking.rs: Concept linking (building task chains, word chains)
│   ├── parsing_task.rs: Narsese task parsing function
│   ├── process_direct.rs: Control process involving "direct inference"
│   ├── process_reason.rs: Control process involving "concept inference"
│   ├── work_cycle.rs: Work cycle control
│   └── ...
├── reasoner: The definition of the reasoner itself and its external API
│   ├── definition.rs: Data structure definition
│   ├── derivation_datas.rs: Work cycle control
│   ├── report.rs: Output report function
│   ├── serde.rs: Serialization and deserialization function
│   ├── vm_api.rs: NAVM virtual machine API
│   └── ...
├── parameters.rs: Hyper-parameters of the reasoner
└── ...
```

#### External Application Interface

NAVM Virtual Machine Self-implemented Implementation `src/vm`: Provides a self-implemented NAVM implementation based on the "reasoner" and NAVM API

```plaintext
vm
├── alpha: Self-contained kernel "Alpha" of the virtual machine
│   ├── cmd_dispatch: NAVM instruction dispatch
│   │   ├── cmd_hlp.rs: Handling instruction `HLP`
│   │   ├── cmd_inf.rs: Handling instruction `INF`
│   │   ├── cmd_loa.rs: Handling instruction `LOA`
│   │   ├── cmd_sav.rs: Handling instruction `SAV`
│   │   └── mod.rs: Top-level dispatch function
│   ├── io: Input and output at the virtual machine level, corresponding to the original OpenNARS channel mechanism
│   │   ├── _channel.rs: Abstract traits related to "channels"
│   │   ├── channel_in.rs: Preliminary implementation of input channels
│   │   ├── channel_out.rs: Preliminary implementation of output channels
│   │   ├── channels.rs: Managing input and output channels on top of the reasoner
│   │   ├── handle_io.rs: Actual logic of channel input and output
│   │   ├── input_channel.rs: Abstract traits of input channels
│   │   ├── output_channel.rs: Abstract traits of output channels
│   │   └── ...
│   ├── launcher.rs: Virtual machine launcher
│   ├── runtime.rs: Virtual machine runtime
│   └── ...
└── ...
```

### Language Features Used

📝Main Rust language features involved in the code (some of which may be challenging to understand):

- Pattern matching
- Generics (static dispatch)
- Traits (abstract interfaces)
- Closures (for simplifying temporary reusable code)
- Modules
- Macros (declarative macros)

⚠️Features that may be difficult to understand:

- `unsafe` code (mutable references for compound terms)
- Trait objects (dynamic dispatch)

### Potential Application Areas

- Web development: Running NARS in the browser, facilitating the integration of NARS into internet applications
- Embedded systems: Running NARS on embedded devices for small kernel, strong specialization scenarios
- Industrial applications: Running NARS on industrial equipment for high performance, low operational overhead scenarios

## Contribution Guidelines

### Project Branch Status

- `main`: The base version, with long-term support
- `dev`: The development version, under continuous development (ℹ️**PRs are mainly merged into this branch**)
- `dev-XXX`: Derivative branches for learning or experimentation
- `archive-XXX`: Archived branches, providing long-term stable and unchanging code
- `debug-XXX`: Temporary branches for solving problems/bugs

### Ways to Contribute

- [Github Issues](https://github.com/ARCJ137442/Narust-158/issues): Report issues, suggestions, bugs
- [GitHub Pull Request](https://github.com/ARCJ137442/Narust-158/pulls): Contribute code directly to the project

## Engineering Log

### [2024-08-19 00:28:53]

📌 Guiding Principle: **Stabilize the Foundation, Extend the Branches**

1. 📍 Provide an external version close to the **declarative core** (LTS Long-term support)
    - 🏗️ Make a solid contribution to the entire NARS engineering ecosystem
    - 🔦 Provide a cornerstone for future generations (especially Rustaceans) to study NARS in engineering
2. 💡 On the foundation of the 'basic version', given my familiarity with Rust, continue to extend and expand, exploring its unique research and application value
    - 🔬 Research: With my familiarity with the core, explore the direction of NAL 7~9 'event reasoning' and 'process reasoning'
    - 🪛 Application: Leverage the advantages of Rust programs in performance, safety, and integrability to explore its application potential in high-performance industrial production, embedded systems, the internet, and other areas where Rust excels

### [2024-06-14 00:41:48]

📌 Core Principle: **Refactor First, Migrate Next, Then Stand Alone**

1. 💡 First, based on the well-established Java development environment, use 'synonymous refactoring' to make the most of Java's development speed and rapid verification advantages
    - ❗ Refactoring is not terrible, what's terrible is 'migrating halfway and having to start over again'
2. 🚚 Before migrating, clarify and **stabilize** the nullability, mutability, reference sharing, and variable ownership of each class, interface, field, and method in Java
    - 🎯 Strive for 'multiple verifications, one migration, one stability'
3. 🏗️ After migrating to Rust and achieving basic stability, start to align the existing test toolchain, and open source it after **aligning with OpenNARS 1.5.8**
    - 🎯 Publicly post, striving to complete before July
