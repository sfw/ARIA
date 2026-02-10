# ARIA Formal Grammar Specification

## Notation

This grammar uses Extended Backus-Naur Form (EBNF) with the following conventions:

- `'text'` — Terminal string
- `NAME` — Non-terminal
- `A B` — Sequence
- `A | B` — Alternation
- `A?` — Optional (zero or one)
- `A*` — Repetition (zero or more)
- `A+` — Repetition (one or more)
- `(A B)` — Grouping
- `[a-z]` — Character range
- `// comment` — Grammar comment

---

## 1. Lexical Grammar

### 1.1 Whitespace and Comments

```ebnf
WHITESPACE = (' ' | '\t' | '\n' | '\r')+

COMMENT = LINE_COMMENT | BLOCK_COMMENT
LINE_COMMENT = '//' (!'\n')* '\n'
BLOCK_COMMENT = '/*' (!'*/')* '*/'

DOC_COMMENT = '///' (!'\n')* '\n'
MODULE_DOC = '//!' (!'\n')* '\n'
```

### 1.2 Identifiers

```ebnf
IDENT = IDENT_START IDENT_CONTINUE*
IDENT_START = [a-zA-Z_] | UNICODE_LETTER
IDENT_CONTINUE = IDENT_START | [0-9]

RAW_IDENT = 'r#' IDENT

// Reserved keywords (cannot be identifiers)
KEYWORD = 'fn' | 'struct' | 'enum' | 'trait' | 'impl' | 'type'
        | 'const' | 'static' | 'let' | 'mut' | 'pub' | 'mod'
        | 'use' | 'as' | 'self' | 'Self'
        | 'if' | 'else' | 'match' | 'for' | 'while' | 'loop'
        | 'break' | 'continue' | 'return' | 'yield' | 'await' | 'async'
        | 'true' | 'false' | 'none'
        | 'unsafe' | 'move' | 'ref'
        | 'where' | 'in' | 'extern' | 'dyn'
```

### 1.3 Literals

```ebnf
LITERAL = INT_LITERAL | FLOAT_LITERAL | STRING_LITERAL
        | CHAR_LITERAL | BOOL_LITERAL | NONE_LITERAL

// Integers
INT_LITERAL = DEC_LITERAL | HEX_LITERAL | OCT_LITERAL | BIN_LITERAL
DEC_LITERAL = DEC_DIGIT (DEC_DIGIT | '_')* INT_SUFFIX?
HEX_LITERAL = '0x' HEX_DIGIT (HEX_DIGIT | '_')* INT_SUFFIX?
OCT_LITERAL = '0o' OCT_DIGIT (OCT_DIGIT | '_')* INT_SUFFIX?
BIN_LITERAL = '0b' BIN_DIGIT (BIN_DIGIT | '_')* INT_SUFFIX?

DEC_DIGIT = [0-9]
HEX_DIGIT = [0-9a-fA-F]
OCT_DIGIT = [0-7]
BIN_DIGIT = [01]

INT_SUFFIX = 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
           | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
           | 'isize' | 'usize'

// Floats
FLOAT_LITERAL = DEC_LITERAL '.' DEC_LITERAL EXPONENT? FLOAT_SUFFIX?
              | DEC_LITERAL EXPONENT FLOAT_SUFFIX?
              | DEC_LITERAL FLOAT_SUFFIX

EXPONENT = ('e' | 'E') ('+' | '-')? DEC_LITERAL
FLOAT_SUFFIX = 'f32' | 'f64'

// Strings
STRING_LITERAL = '"' STRING_CONTENT* '"'
               | 'r' RAW_STRING

STRING_CONTENT = !'"' | ESCAPE_SEQ | INTERPOLATION
ESCAPE_SEQ = '\\' ('n' | 'r' | 't' | '\\' | '"' | '\'' | '0'
           | 'x' HEX_DIGIT HEX_DIGIT
           | 'u{' HEX_DIGIT+ '}')

INTERPOLATION = '{' EXPR '}'

RAW_STRING = '"' RAW_STRING_CONTENT* '"'
           | '#'+ '"' RAW_STRING_BODY '"' '#'+

// Characters
CHAR_LITERAL = '\'' (CHAR_CONTENT | ESCAPE_SEQ) '\''
CHAR_CONTENT = !('"' | '\\')

// Booleans
BOOL_LITERAL = 'true' | 'false'

// None
NONE_LITERAL = 'none'
```

### 1.4 Operators and Punctuation

```ebnf
// Arithmetic
PLUS = '+'
MINUS = '-'
STAR = '*'
SLASH = '/'
PERCENT = '%'

// Comparison
EQ = '=='
NE = '!='
LT = '<'
LE = '<='
GT = '>'
GE = '>='

// Logical
AND = '&&'
OR = '||'
NOT = '!'

// Bitwise
AMP = '&'
PIPE = '|'
CARET = '^'
SHL = '<<'
SHR = '>>'

// Assignment
ASSIGN = '='
PLUS_ASSIGN = '+='
MINUS_ASSIGN = '-='
STAR_ASSIGN = '*='
SLASH_ASSIGN = '/='

// Punctuation
LPAREN = '('
RPAREN = ')'
LBRACKET = '['
RBRACKET = ']'
LBRACE = '{'
RBRACE = '}'
COMMA = ','
COLON = ':'
SEMI = ';'
DOT = '.'
DOTDOT = '..'
DOTDOTEQ = '..='
ARROW = '->'
FAT_ARROW = '=>'
QUESTION = '?'
PIPE_PIPE = '|>'
DOUBLE_COLON = '::'
AT = '@'
HASH = '#'
```

---

## 2. Syntactic Grammar

### 2.1 Source File

```ebnf
SourceFile = ModuleDoc? Item*

ModuleDoc = MODULE_DOC+

Item = OuterAttr* (VisItem | MacroItem)

VisItem = Visibility? (
    FnDef
  | StructDef
  | EnumDef
  | TraitDef
  | ImplBlock
  | TypeAlias
  | ConstDef
  | StaticDef
  | ModDef
  | UseDef
  | ExternBlock
)
```

### 2.2 Visibility

```ebnf
Visibility = 'pub' VisibilityRestriction?

VisibilityRestriction = '(' VisibilityScope ')'

VisibilityScope = 'crate'
                | 'super'
                | 'self'
                | 'in' Path
```

### 2.3 Functions

```ebnf
FnDef = FnSig Block

FnSig = 'async'? 'unsafe'? 'fn' IDENT GenericParams? '(' FnParams? ')' ReturnType? WhereClause?

FnParams = FnParam (',' FnParam)* ','?

FnParam = OuterAttr* ('mut'? IDENT ':')? Type ('=' Expr)?

ReturnType = '->' Type
```

### 2.4 Structs

```ebnf
StructDef = 'struct' IDENT GenericParams? StructBody WhereClause?

StructBody = StructFields
           | TupleFields ';'
           | ';'

StructFields = '{' (StructField (',' StructField)* ','?)? '}'

StructField = OuterAttr* Visibility? IDENT ':' Type

TupleFields = '(' (TupleField (',' TupleField)* ','?)? ')'

TupleField = OuterAttr* Visibility? Type
```

### 2.5 Enums

```ebnf
EnumDef = 'enum' IDENT GenericParams? WhereClause? '{' EnumVariants? '}'

EnumVariants = EnumVariant (',' EnumVariant)* ','?

EnumVariant = OuterAttr* IDENT EnumVariantData?

EnumVariantData = StructFields
                | TupleFields
                | '=' Expr
```

### 2.6 Traits

```ebnf
TraitDef = 'unsafe'? 'trait' IDENT GenericParams? TypeBounds? WhereClause? '{' TraitItem* '}'

TraitItem = OuterAttr* (
    TraitFn
  | TraitConst
  | TraitType
)

TraitFn = FnSig (';' | Block)

TraitConst = 'const' IDENT ':' Type ('=' Expr)? ';'

TraitType = 'type' IDENT GenericParams? TypeBounds? ('=' Type)? ';'
```

### 2.7 Implementations

```ebnf
ImplBlock = 'unsafe'? 'impl' GenericParams? Type ('for' Type)? WhereClause? '{' ImplItem* '}'

ImplItem = OuterAttr* Visibility? (
    FnDef
  | ConstDef
  | TypeAlias
)
```

### 2.8 Type Aliases

```ebnf
TypeAlias = 'type' IDENT GenericParams? WhereClause? '=' Type ';'
```

### 2.9 Constants and Statics

```ebnf
ConstDef = 'const' IDENT ':' Type '=' Expr ';'

StaticDef = 'static' 'mut'? IDENT ':' Type '=' Expr ';'
```

### 2.10 Modules

```ebnf
ModDef = 'mod' IDENT (';' | '{' Item* '}')

UseDef = 'use' UsePath ';'

UsePath = ('::')? UsePathSegment ('::' UsePathSegment)*

UsePathSegment = IDENT
               | '*'
               | '{' UsePathList '}'
               | IDENT 'as' IDENT

UsePathList = UsePath (',' UsePath)* ','?
```

### 2.11 Extern Blocks

```ebnf
ExternBlock = 'extern' STRING_LITERAL? '{' ExternItem* '}'

ExternItem = OuterAttr* Visibility? (ExternFn | ExternStatic)

ExternFn = 'fn' IDENT '(' FnParams? ')' ReturnType? ';'

ExternStatic = 'static' 'mut'? IDENT ':' Type ';'
```

---

## 3. Types

```ebnf
Type = ParenType
     | TupleType
     | ArrayType
     | SliceType
     | ReferenceType
     | PointerType
     | FnType
     | PathType
     | InferredType
     | NeverType

ParenType = '(' Type ')'

TupleType = '(' (Type (',' Type)+ ','?)? ')'

ArrayType = '[' Type ';' Expr ']'

SliceType = '[' Type ']'

ReferenceType = '&' 'mut'? Type

PointerType = '*' ('const' | 'mut') Type

FnType = 'fn' '(' TypeList? ')' ReturnType?

PathType = Path GenericArgs?

InferredType = '_'

NeverType = 'Never'

TypeList = Type (',' Type)* ','?
```

### 3.1 Generics

```ebnf
GenericParams = '[' GenericParam (',' GenericParam)* ','? ']'

GenericParam = TypeParam
             | ConstParam

TypeParam = IDENT TypeBounds?

ConstParam = 'const' IDENT ':' Type

GenericArgs = '[' GenericArg (',' GenericArg)* ','? ']'

GenericArg = Type
           | Expr  // for const generics

TypeBounds = ':' TypeBound ('+' TypeBound)*

TypeBound = Path GenericArgs?
          | '(' Type ')'

WhereClause = 'where' WherePredicate (',' WherePredicate)* ','?

WherePredicate = Type ':' TypeBounds
```

---

## 4. Expressions

### 4.1 Expression Precedence

```ebnf
// Precedence from lowest to highest
Expr = AssignExpr

AssignExpr = OrExpr (AssignOp OrExpr)?
AssignOp = '=' | '+=' | '-=' | '*=' | '/=' | '%=' | '&=' | '|=' | '^=' | '<<=' | '>>='

OrExpr = AndExpr ('||' AndExpr)*

AndExpr = CompareExpr ('&&' CompareExpr)*

CompareExpr = BitOrExpr (CompareOp BitOrExpr)?
CompareOp = '==' | '!=' | '<' | '<=' | '>' | '>='

BitOrExpr = BitXorExpr ('|' BitXorExpr)*

BitXorExpr = BitAndExpr ('^' BitAndExpr)*

BitAndExpr = ShiftExpr ('&' ShiftExpr)*

ShiftExpr = AddExpr (('<<' | '>>') AddExpr)*

AddExpr = MulExpr (('+' | '-') MulExpr)*

MulExpr = UnaryExpr (('*' | '/' | '%') UnaryExpr)*

UnaryExpr = ('-' | '!' | '&' | '&' 'mut' | '*') UnaryExpr
          | PostfixExpr

PostfixExpr = PrimaryExpr PostfixOp*

PostfixOp = '.' IDENT
          | '.' INT_LITERAL
          | '(' CallArgs? ')'
          | '[' Expr ']'
          | '?'
          | '|>' IDENT
          | 'as' Type
```

### 4.2 Primary Expressions

```ebnf
PrimaryExpr = LITERAL
            | Path
            | '(' Expr ')'
            | '(' Expr (',' Expr)+ ','? ')'  // Tuple
            | '[' ArrayElems? ']'
            | Block
            | IfExpr
            | MatchExpr
            | ForExpr
            | WhileExpr
            | LoopExpr
            | ReturnExpr
            | BreakExpr
            | ContinueExpr
            | ClosureExpr
            | AsyncExpr
            | AwaitExpr
            | StructExpr

ArrayElems = Expr (',' Expr)* ','?
           | Expr ';' Expr

CallArgs = Expr (',' Expr)* ','?
         | IDENT ':' Expr (',' IDENT ':' Expr)* ','?
```

### 4.3 Control Flow Expressions

```ebnf
IfExpr = 'if' Expr Block ('else' (IfExpr | Block))?

MatchExpr = 'match' Expr '{' MatchArm* '}'

MatchArm = OuterAttr* Pattern MatchGuard? '=>' (Expr ','? | Block)

MatchGuard = 'if' Expr

ForExpr = 'for' Pattern 'in' Expr Block

WhileExpr = 'while' Expr Block
          | 'while' 'let' Pattern '=' Expr Block

LoopExpr = Label? 'loop' Block

Label = '\'' IDENT ':'

ReturnExpr = 'return' Expr?

BreakExpr = 'break' Label? Expr?

ContinueExpr = 'continue' Label?
```

### 4.4 Closures and Async

```ebnf
ClosureExpr = 'move'? '|' ClosureParams? '|' ReturnType? (Expr | Block)

ClosureParams = ClosureParam (',' ClosureParam)* ','?

ClosureParam = Pattern (':' Type)?

AsyncExpr = 'async' 'move'? Block

AwaitExpr = Expr '.' 'await'
```

### 4.5 Struct Expressions

```ebnf
StructExpr = Path '{' StructExprFields? '}'

StructExprFields = StructExprField (',' StructExprField)* (',' StructExprBase)? ','?
                 | StructExprBase

StructExprField = IDENT (':' Expr)?

StructExprBase = '..' Expr
```

---

## 5. Patterns

```ebnf
Pattern = PatternNoOr ('|' PatternNoOr)*

PatternNoOr = LiteralPattern
            | IdentPattern
            | WildcardPattern
            | RestPattern
            | RangePattern
            | RefPattern
            | StructPattern
            | TuplePattern
            | SlicePattern
            | PathPattern
            | MacroPattern

LiteralPattern = '-'? LITERAL

IdentPattern = 'ref'? 'mut'? IDENT ('@' Pattern)?

WildcardPattern = '_'

RestPattern = '..'

RangePattern = RangePatternBound? '..' '='? RangePatternBound?

RangePatternBound = LiteralPattern | Path

RefPattern = '&' 'mut'? Pattern

StructPattern = Path '{' StructPatternFields? '}'

StructPatternFields = StructPatternField (',' StructPatternField)* (',' RestPattern)? ','?

StructPatternField = IDENT (':' Pattern)?

TuplePattern = '(' PatternList? ')'

SlicePattern = '[' PatternList? ']'

PatternList = Pattern (',' Pattern)* ','?

PathPattern = Path ('(' PatternList? ')')?
```

---

## 6. Statements

```ebnf
Block = '{' Statement* Expr? '}'

Statement = ';'
          | Item
          | LetStatement
          | ExprStatement

LetStatement = OuterAttr* 'let' Pattern (':' Type)? ('=' Expr)? ';'

ExprStatement = Expr ';'
              | ExprWithBlock ';'?

ExprWithBlock = Block
              | IfExpr
              | MatchExpr
              | ForExpr
              | WhileExpr
              | LoopExpr
              | AsyncExpr
```

---

## 7. Attributes

```ebnf
Attribute = InnerAttr | OuterAttr

InnerAttr = '#!' '[' AttrContent ']'

OuterAttr = '#' '[' AttrContent ']'

AttrContent = Path AttrInput?

AttrInput = '(' TokenTree* ')'
          | '=' LITERAL
```

---

## 8. Paths

```ebnf
Path = '::'? PathSegment ('::' PathSegment)*

PathSegment = 'self'
            | 'super'
            | 'crate'
            | IDENT GenericArgs?
            | '<' Type ('as' Path)? '>'
```

---

## 9. Grammar Notes for AI Code Generation

### 9.1 Unambiguous Constructs

The grammar is designed to be LL(1) or nearly so, enabling:
- Predictable parsing without backtracking
- DFA-based token validation during generation
- Clear error recovery points

### 9.2 Key Disambiguations

```ebnf
// '<' is always followed by type context or comparison
GenericStart = '<' (Type | IDENT ':')  // Not comparison
Comparison = Expr '<' Expr              // Comparison

// Block vs struct literal
BlockExpr = '{' Statement* Expr? '}'    // Has statements or trailing expr
StructLit = Path '{' Fields '}'         // Has Path prefix

// Closure vs or-pattern
Closure = '|' Params '|' Body           // Has body
OrPattern = Pat '|' Pat                 // In pattern context
```

### 9.3 Recovery Points

```ebnf
// Statement boundaries
StatementEnd = ';' | '}'

// Item boundaries
ItemStart = 'fn' | 'struct' | 'enum' | 'trait' | 'impl' | 'mod' | 'use' | 'pub'

// Expression boundaries
ExprEnd = ';' | ')' | ']' | '}' | ','
```
