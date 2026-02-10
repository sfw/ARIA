# AI Code Generation Research for Language Design

## Executive Summary

This document presents research on how generative AI (LLMs) generates code and what language features optimize for AI-assisted programming. The goal is to inform the design of a programming language that AI can generate correctly and efficiently.

**Key Finding:** Type errors account for 33.6% of failed LLM-generated programs. Strong static typing with inference, combined with structured error messages, dramatically improves AI code generation accuracy.

---

## 1. Patterns LLMs Struggle With

### Type-Related Errors (33.6% of failures)

- Treating undeclared variables as declared
- Making simultaneous mutable borrows
- Using the same variable as both mutable and immutable
- Incorrectly treating private variables as public
- GitHub Copilot: 24% of suggestions result in compilation errors (mainly type errors)

### Memory Management and Ownership

For Rust specifically:
- **94.8% of failures are compilation errors** in translation benchmarks (61.9% are dependency resolution; lifetime/ownership is a secondary layer)
- Borrow checker and module system create particular difficulty
- Models cannot just learn syntax—they must internalize ownership logic as hard constraints

### API Hallucinations

A major issue identified across LLMs:
- Parameter errors
- Improper guard conditions
- Similar-but-incorrect/deprecated API usage
- StarCoder: **43.3% hallucinated parameter misuses in Java**
- Analysis of 576,000 samples: **5.2% hallucinated packages in Python, 21.7% in JavaScript**

### Complex Multi-Step Problems

- Original Codex: only 28.8% of HumanEval tasks solved on first attempt
- GPT-4: over 90% on easier problems
- Still struggle with complex multi-step problem descriptions
- Tend to generate common patterns rather than optimal solutions

---

## 2. Language Features That Improve AI Code Generation

### Strong Type Systems

MIT research demonstrates that guiding LLMs toward programming language rules significantly improves accuracy:
- **Small LLMs can outperform much larger models** when type-constrained
- Type-constrained decoding **reduces compilation errors by more than half**
- Increases functional correctness by 3.5%-5.5%
- Enhances correct repair of non-compiling code by **37% on average**

### Formal Verification Features

Languages designed for formal verification (like Dafny) show promising results:
- Best LLMs solved **over 60% of vericoding problems**
- DafnyPro achieves **86% correct proofs** with Claude 3.5 Sonnet
- Key insight: When code cannot be proven correct, it cannot be generated

### Grammar Constraints

SynCode (using context-free grammar to constrain outputs):
- Achieves **96.07% reduction in syntax errors**
- Uses DFA mask stores based on language grammar terminals

---

## 3. How LLMs Handle Different Language Aspects

### Type Systems

- **Static typing improves reliability**: TypeScript performs well due to structural constraints
- **Type annotations have mixed effects**: Improve well-typedness but are language-dependent
- **Dynamic languages pose challenges**: 30% of developer queries relate to type issues

### Memory Management

Rust's strict model creates a **natural feedback loop**:
- If LLM-generated code has memory issues, compiler rejects it
- Unlike C++ where subtle memory leaks may go undetected
- Challenge: Model must learn the vast landscape of incorrect patterns

### Error Handling

Research on AI-generated exception handling found:
- **16 of 19 error causes** belong to LLM incapability
- Only 3 causes are amenable to automated fixing:
  - Inconsistent Indentation
  - Function Overflow
  - Missing Import
- **45% of AI-generated code contains security vulnerabilities**

---

## 4. Token Efficiency and AI-Friendly Syntax

### Token Reduction Strategies

Research on AI-oriented grammar (SimPy) demonstrates:
- Remove redundant delimiters (consecutive newlines, unnecessary whitespace)
- Simplify keywords, operators, and delimiters to compact forms
- Results: **13.5% token reduction for CodeLlama, 10.4% for GPT-4**

### Formatting Costs

Code formatting research shows:
- C++ achieves **31.12% code token reduction** by removing formatting
- C# achieves **25.26% reduction**
- Python only **6.51% reduction** (formatting is semantic)
- Newlines contribute **14.6%-17.5%** of tokens

### TOON Format Example

Token-Oriented Object Notation achieves **30-60% fewer tokens than JSON** by:
- Eliminating redundant punctuation
- Using indentation-based structure

---

## 5. Language Performance Comparison

### MultiPL-E Benchmark Findings

**Best Performance Languages:**
- JavaScript (matches or exceeds Python)
- TypeScript, C++, Scala (comparable to Python)
- Python (strong training data representation)

**Struggling Languages:**
- OCaml, Racket (limited training data)
- Rust (94.8% of failures are compilation errors — mostly dependency resolution)

### Key Correlations

- Performance correlates with **language popularity** in training data
- Some niche languages perform as well as popular ones
- **No significant difference** between optionally-typed Python and gradually-typed TypeScript

---

## 6. Context Window Considerations

### Current State

- GPT-4 Turbo: 128k tokens
- Claude 2.1: 200k tokens
- Gemini 1.5: 1 million tokens

### Performance Degradation

LongCodeBench research reveals significant issues:
- Claude 3.5 Sonnet: drops from **29% to 3%** as context increases
- Qwen2.5: drops from **70.2% to 40%**
- "Lost in the Middle" problem: models perform best at beginning and end

### Design Implications

- **Modular code design** reduces context requirements
- **Self-contained functions** with clear interfaces help LLMs focus
- **Local dependency resolution** may help
- **Concise syntax** maximizes useful context within token limits

---

## 7. Error Messages That Help LLMs Self-Correct

### Effective Feedback Patterns

LLMLOOP framework achievements:
- First loop ensures all generated code is compilable
- Iterative refinement based on verifier feedback
- Up to **63% pass rates** with carefully structured prompts

### Self-Debug Approaches

Teaching LLMs to debug via:
- Rubber duck debugging (explaining code in natural language)
- Execution result investigation
- Up to **12% improvement** on TransCoder and MBPP benchmarks

### Requirements for Helpful Error Messages

1. **Include original erroneous source code** - vague feedback produces unhelpful responses
2. **Provide specific, actionable information** - stack traces, exception types
3. **Plain language explanations** without excessive jargon
4. **Propose minimal, concrete fixes** rather than entire rewrites

### Multi-Tool Feedback Systems

Combining multiple feedback sources shows dramatic improvements:
- Compiler diagnostics + CodeQL + KLEE symbolic execution
- Security vulnerabilities reduced by **96% for DeepSeek**
- Critical defect rates reduced from **58.55% to 22.19%**

---

## 8. Design Recommendations for AI-Friendly Language

Based on this research, an AI-optimized programming language should:

### Type System
1. **Strong, gradual static typing** with inference
2. Catches errors at compile time while reducing annotation burden
3. Consider refinement types for critical paths

### Verification
4. **Formal verification capabilities** - enable proof-carrying code generation
5. When code cannot be proven correct, it cannot be generated

### Syntax
6. **Token-efficient syntax** - minimize redundant delimiters and formatting
7. Consider significant whitespace vs. braces tradeoff
8. Design grammar amenable to DFA-based constraint checking

### Error Handling
9. **Rich, structured error messages** - include context, locations, suggested fixes
10. Provide multiple feedback sources (type errors, security analysis)

### Architecture
11. **Modular design encouragement** - small, self-contained functions
12. **Local dependency resolution** - reduce context requirements
13. **Stable, well-documented APIs** - reduce hallucination potential

### Memory Safety
14. **Explicit state management** - clear ownership/borrowing semantics
15. Better error messages than Rust when violations occur
16. Natural feedback loop: reject unsafe code at compile time

---

## Sources

- A Survey On Large Language Models For Code Generation (arXiv)
- Towards Advancing Code Generation with Large Language Models (arXiv)
- MIT News: Making AI-generated code more accurate
- Type-Constrained Code Generation with Language Models (arXiv)
- Repository-level Code Translation Benchmark Targeting Rust (arXiv)
- AI Coders Are Among Us: Rethinking Programming Language Grammar (arXiv)
- The Hidden Cost of Readability: Code Formatting and LLM Budget (arXiv)
- LLM Hallucinations in Practical Code Generation (arXiv)
- MultiPL-E Benchmark (GitHub)
- LongCodeBench: Evaluating Coding LLMs at 1M Context Windows (arXiv)
- LLMLOOP: Improving LLM-Generated Code
- Teaching Large Language Models to Self-Debug (arXiv)
- SynCode: Grammar-Augmented LLM Code Generation (arXiv)
