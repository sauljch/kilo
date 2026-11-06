# Kilo

This repository contains _my_ Rust implementation of [antirez's Kilo text editor](https://antirez.com/news/108), traditionally written in C. This is being implemented by following along with the tutorial at https://viewsourcecode.org/snaptoken/kilo/. I've tried to implement the editor in a manner similar to what's described in the tutorial, but due to the differences between Rust and C – or more likely due to my ignorance about the features of Rust – a 1:1 recreation has not been feasible. Either way, I will try to _refrain_ from using anything other than Rust's standard library and the necessary C libraries for this project.

_Disclaimer: This editor is only guaranteed to work on macOS, and even then, probably not all versions._

## Implementation

As of commit 059dd22 (main), the main loop looks like:

```mermaid
flowchart TD
    INIT@{ shape: subproc, label: "Initialize editor (1)" }
    ARGS["Collect arguments"]
    INPUT@{ shape: diamond, label: "File passed?" }
    OPEN@{ shape: subproc, label: "Open file (2)" }
    LOOP@{ shape: subproc, label: "Editor loop (3)" }

    INIT --> ARGS
    ARGS --> INPUT
    INPUT -- "Yes" --> OPEN
    INPUT -- "No" --> LOOP
    OPEN --> LOOP
```

All the interesting things happen in blocks (1), (2), and (3).

### 1. Initializing the editor

The editor initialization happens first and is described in `EditorConfig::new()`. That process looks like:

```mermaid
flowchart LR
    WINDOW["Get window size"]
    INSTANCE@{ shape: subproc, label: "Create EditorConfig" }
    ALT@{ shape: subproc, label: "Enter alternate screen" }
    CLEAR@{ shape: subproc, label: "Clear window" }

    WINDOW --> INSTANCE
    INSTANCE --> ALT
    ALT --> CLEAR
```

### 2. Opening a file

The process for opening a file happens in `EditorConfig::open()` and looks like:

```mermaid
flowchart TD
    OPEN["Open file"]
    LINE@{ shape: diamond, label: "Line exists?" }
    REFRESH@{ shape: subproc, label: "Refresh screen" }
    LOAD@{ shape: subproc, label: "Load line into editor row buffer" }

    OPEN --> LINE
    LINE -- "No" --> REFRESH
    LINE -- "Yes" --> LOAD
    LOAD -- "Try next line" --> LINE
```

### 3. Core editor loop

The core editor loop occurs within `main()` and only consists of two processes:

```mermaid
flowchart LR
    KEYPRESS@{ shape: subproc, label: "Process keypress" }
    REFRESH@{ shape: subproc, label: "Refresh screen" }

    KEYPRESS --> REFRESH
    REFRESH --> KEYPRESS
```
