# harness cli

A set of CLIs for harness usage.

LLMs are familiar with the cli tools like `rg`, `nl`, `find`, `sed` to grep what
they wants in the codespace. They are cool tools with agent-free output, but
they are created for text process, nor code analyze. UNIX tools provide precise
and focused functions, that are easy for agent to use, but also providing less
information make the agent use more turns to get what they want

For example, `grep` can search any patterns but usually just gets approximate
line number and call the `Read` tools or `sed` randomly that has introduced more
context usage

This repo introduce more find-grained cli tools to provide interfaces to get
higher level information than the traditional UNIX tools.

## Content

- `crate/outlines`: Tool to provide file outline information.
