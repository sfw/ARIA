# ARIA v2 Formal Grammar

## Notation

- `'text'` — Terminal
- `NAME` — Non-terminal
- `A B` — Sequence
- `A | B` — Alternative
- `A?` — Optional
- `A*` — Zero or more
- `A+` — One or more
- `INDENT` — Increase indentation
- `DEDENT` — Decrease indentation
- `NEWLINE` — Line break
- `SAME` — Same indentation level

---

## 1. Lexical Grammar

### 1.1 Keywords

```
KEYWORD = 'f' | 's' | 'e' | 't' | 'i' | 'm'
        | 'if' | 'then' | 'else'
        | 'for' | 'in' | 'wh' | 'lp'
        | 'br' | 'ct' | 'ret'
        | 'as' | 'aw'
        | 'us' | 'md' | 'pub'
        | 'mut' | 'mv' | 'un'
        | 'type' | 'where'
        | 'T' | 'F' | 'N'
        | 'Some' | 'Ok' | 'Err'
        | 'ok' | 'err' | 'none'
```

### 1.2 Identifiers

```
IDENT = IDENT_START IDENT_CONT*
IDENT_START = [a-zA-Z_]
IDENT_CONT = IDENT_START | [0-9]
```

### 1.3 Literals

```
INT = DEC_INT | HEX_INT | BIN_INT | OCT_INT
DEC_INT = [0-9] ([0-9_])* INT_SUFFIX?
HEX_INT = '0x' [0-9a-fA-F_]+ INT_SUFFIX?
BIN_INT = '0b' [01_]+ INT_SUFFIX?
OCT_INT = '0o' [0-7_]+ INT_SUFFIX?
INT_SUFFIX = 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
           | 'u8' | 'u16' | 'u32' | 'u64' | 'u128'

FLOAT = DEC_INT '.' DEC_INT EXP? FLOAT_SUFFIX?
      | DEC_INT EXP FLOAT_SUFFIX?
EXP = [eE] [+-]? DEC_INT
FLOAT_SUFFIX = 'f32' | 'f64'

STRING = '"' STRING_CHAR* '"'
       | '`' RAW_CHAR* '`'
       | '```' MULTI_CHAR* '```'
STRING_CHAR = ESCAPE | INTERP | [^"\\]
ESCAPE = '\\' [nrt\\'"0] | '\\x' HEX HEX | '\\u{' HEX+ '}'
INTERP = '{' EXPR '}'

CHAR = '\'' (ESCAPE | [^'\\]) '\''

BOOL = 'T' | 'F' | 'true' | 'false'
NONE = 'N' | 'none'
```

### 1.4 Operators

```
OP = '+' | '-' | '*' | '/' | '%'
   | '==' | '!=' | '<' | '<=' | '>' | '>='
   | '&&' | '||' | '!'
   | '&' | '|' | '^' | '<<' | '>>'
   | '=' | ':=' | '+=' | '-=' | '*=' | '/='
   | '?' | '??' | '!'
   | '->' | '=>' | '|>'
   | '..' | '..='
   | '::' | '.' | ','
   | '@'
```

### 1.5 Delimiters

```
DELIM = '(' | ')' | '[' | ']' | '{' | '}'
      | ':' | ';'
```

---

## 2. Indentation Rules

ARIA v2 uses significant indentation:

```
# Block starts after ':' or keyword line
f foo -> Int
    x = 1      # INDENT
    y = 2
    x + y      # DEDENT (implicit)

# Braces override indentation
f foo -> Int { x = 1; y = 2; x + y }

# Continuation: indent continues previous line
long_expr = a + b
    + c + d    # continuation
```

---

## 3. Top Level

```
SourceFile = Item*

Item = Attribute* (
    FnDef
  | StructDef
  | EnumDef
  | TraitDef
  | ImplDef
  | TypeDef
  | UseDef
  | ModDef
)

Attribute = '@' IDENT AttrArgs?
AttrArgs = '(' AttrArgList ')'
AttrArgList = AttrArg (',' AttrArg)*
AttrArg = IDENT ('=' Literal)?
```

---

## 4. Declarations

### 4.1 Functions

```
FnDef = Async? 'f' IDENT GenericParams? Params? ReturnType? Body

Async = 'as'

Params = '(' ParamList? ')'
ParamList = Param (',' Param)*
Param = IDENT ':' Type DefaultValue?
DefaultValue = '=' Expr

ReturnType = '->' Type

Body = '=' Expr
     | NEWLINE INDENT Statement+ DEDENT
     | '{' Statement* Expr? '}'
```

### 4.2 Structs

```
StructDef = 's' IDENT GenericParams? StructBody

StructBody = TupleFields
           | NEWLINE INDENT StructField+ DEDENT
           | '{' StructFieldList '}'

TupleFields = '(' TypeList ')'

StructField = IDENT ':' Type DefaultValue? NEWLINE

StructFieldList = StructFieldInline (',' StructFieldInline)*
StructFieldInline = IDENT ':' Type DefaultValue?
```

### 4.3 Enums

```
EnumDef = 'e' IDENT GenericParams? EnumBody

EnumBody = '=' EnumVariantInline ('|' EnumVariantInline)*
         | NEWLINE INDENT EnumVariant+ DEDENT
         | '{' EnumVariantList '}'

EnumVariant = IDENT EnumVariantData? NEWLINE

EnumVariantData = '(' FieldList ')'
                | '(' TypeList ')'

EnumVariantInline = IDENT EnumVariantData?
EnumVariantList = EnumVariantInline (',' EnumVariantInline)*
```

### 4.4 Traits

```
TraitDef = 't' IDENT GenericParams? SuperTraits? TraitBody

SuperTraits = ':' TypeBound ('+' TypeBound)*

TraitBody = NEWLINE INDENT TraitItem+ DEDENT
          | '{' TraitItem* '}'

TraitItem = TypeAlias
          | FnSig
          | FnDef
```

### 4.5 Implementations

```
ImplDef = 'i' GenericParams? Type ForClause? WhereClause? ImplBody

ForClause = 'for' Type

ImplBody = NEWLINE INDENT ImplItem+ DEDENT
         | '{' ImplItem* '}'

ImplItem = TypeAlias
         | FnDef
```

### 4.6 Type Alias

```
TypeDef = 'type' IDENT GenericParams? '=' Type
```

### 4.7 Use

```
UseDef = 'us' UsePath

UsePath = PathPrefix? UseTree

PathPrefix = ('std' | 'crate' | 'self' | 'super') '.'

UseTree = IDENT ('.' UseTree)?
        | IDENT '->' IDENT
        | '{' UseTreeList '}'
        | '*'

UseTreeList = UseTree (',' UseTree)*
```

### 4.8 Module

```
ModDef = 'md' IDENT ModBody?

ModBody = NEWLINE INDENT Item+ DEDENT
        | '{' Item* '}'
```

---

## 5. Types

```
Type = TypePath
     | TypeShortcut
     | TupleType
     | FnType
     | RefType
     | PtrType

TypePath = IDENT GenericArgs?
         | IDENT '.' TypePath

GenericArgs = '[' TypeArgList ']'
TypeArgList = TypeArg (',' TypeArg)*
TypeArg = Type | Expr

TypeShortcut = ListType
             | MapType
             | SetType
             | ArrayType
             | OptionType
             | ResultType

ListType = '[' Type ']'
MapType = '{' Type ':' Type '}'
SetType = '{' Type '}'
ArrayType = '[' Type ';' Expr ']'
OptionType = Type '?'
ResultType = Type '!' Type?

TupleType = '(' TypeList? ')'
TypeList = Type (',' Type)*

FnType = Type '->' Type
       | '(' TypeList? ')' '->' Type

RefType = '&' 'mut'? Type
PtrType = '*' 'mut'? Type
```

---

## 6. Generics

```
GenericParams = '[' GenericParamList ']'
GenericParamList = GenericParam (',' GenericParam)*
GenericParam = TypeParam | ConstParam

TypeParam = IDENT TypeBounds?
ConstParam = IDENT ':' Type

TypeBounds = ':' TypeBound ('+' TypeBound)*
TypeBound = TypePath

WhereClause = 'where' WherePred (NEWLINE WherePred)*
WherePred = Type ':' TypeBounds
```

---

## 7. Expressions

### 7.1 Precedence (lowest to highest)

```
Expr = AssignExpr

AssignExpr = PipeExpr (AssignOp PipeExpr)?
AssignOp = '=' | ':=' | '+=' | '-=' | '*=' | '/='

PipeExpr = OrExpr ('|' OrExpr)*
        | OrExpr '|>' OrExpr

OrExpr = AndExpr ('||' AndExpr)*

AndExpr = CmpExpr ('&&' CmpExpr)*

CmpExpr = BitOrExpr (CmpOp BitOrExpr)?
CmpOp = '==' | '!=' | '<' | '<=' | '>' | '>='

BitOrExpr = BitXorExpr ('|' BitXorExpr)*

BitXorExpr = BitAndExpr ('^' BitAndExpr)*

BitAndExpr = ShiftExpr ('&' ShiftExpr)*

ShiftExpr = AddExpr (('<<' | '>>') AddExpr)*

AddExpr = MulExpr (('+' | '-') MulExpr)*

MulExpr = UnaryExpr (('*' | '/' | '%') UnaryExpr)*

UnaryExpr = ('-' | '!' | '&' | '&' 'mut' | '*') UnaryExpr
          | PostfixExpr

PostfixExpr = PrimaryExpr Postfix*

Postfix = '.' IDENT
        | '.' INT
        | '(' ArgList? ')'
        | '[' Expr ']'
        | '?'
        | '??'
```

### 7.2 Primary Expressions

```
PrimaryExpr = Literal
            | IDENT
            | Path
            | '(' Expr ')'
            | '(' ExprList ')'
            | '[' ArrayElems? ']'
            | '{' MapElems? '}'
            | Block
            | IfExpr
            | MatchExpr
            | ForExpr
            | WhileExpr
            | LoopExpr
            | Closure
            | AsyncExpr
            | AwaitExpr
            | StructExpr
            | ReturnExpr
            | BreakExpr
            | ContinueExpr

Literal = INT | FLOAT | STRING | CHAR | BOOL | NONE

Path = IDENT ('.' IDENT)*

ArrayElems = Expr (',' Expr)* ','?
           | Expr ';' Expr

MapElems = MapElem (',' MapElem)* ','?
MapElem = Expr ':' Expr

ExprList = Expr (',' Expr)+ ','?

ArgList = Arg (',' Arg)* ','?
Arg = (IDENT ':')? Expr
```

### 7.3 Control Flow

```
IfExpr = 'if' Expr 'then' Expr 'else' Expr
       | 'if' Expr NEWLINE INDENT Block DEDENT ElseClause?
       | 'if' Expr '{' Block '}' ElseClause?

ElseClause = 'else' 'if' Expr Block ElseClause?
           | 'else' Block

MatchExpr = 'm' Expr NEWLINE INDENT MatchArm+ DEDENT
          | 'm' Expr '{' MatchArm* '}'
          | Expr '|' InlineMatchArm ('|' InlineMatchArm)*

MatchArm = Pattern Guard? '->' Expr NEWLINE
         | Pattern Guard? '->' NEWLINE INDENT Block DEDENT

InlineMatchArm = Pattern Guard? '->' Expr

Guard = 'if' Expr

ForExpr = 'for' Pattern 'in' Expr Block

WhileExpr = 'wh' Expr Block
          | 'wh' Pattern '=' Expr Block

LoopExpr = 'lp' Block

Block = NEWLINE INDENT Statement+ DEDENT
      | '{' Statement* Expr? '}'
```

### 7.4 Closures and Async

```
Closure = '|' ParamList? '|' ReturnType? ClosureBody
        | '||' ClosureBody
        | FieldShorthand
        | OpShorthand

ClosureBody = Expr
            | Block

FieldShorthand = '.' IDENT

OpShorthand = '(' Op Expr ')'
            | '(' Expr Op ')'

AsyncExpr = 'as' Block

AwaitExpr = 'aw' Expr
```

### 7.5 Other Expressions

```
StructExpr = TypePath '(' FieldInitList? ')'
           | TypePath '{' FieldInitList? '}'

FieldInitList = FieldInit (',' FieldInit)* (',' StructBase)? ','?
FieldInit = IDENT (':' Expr)?
StructBase = '..' Expr

ReturnExpr = 'ret' Expr?
BreakExpr = 'br' Label? Expr?
ContinueExpr = 'ct' Label?

Label = '\'' IDENT
```

---

## 8. Patterns

```
Pattern = PatternNoOr ('|' PatternNoOr)*

PatternNoOr = LiteralPat
            | IdentPat
            | WildcardPat
            | TuplePat
            | ListPat
            | StructPat
            | RangePat
            | RefPat

LiteralPat = '-'? Literal

IdentPat = 'mut'? IDENT ('@' Pattern)?

WildcardPat = '_'

TuplePat = '(' PatternList? ')'

ListPat = '[' PatternList? ']'

StructPat = TypePath '(' PatternFieldList? ')'
          | TypePath '{' PatternFieldList? '}'

PatternFieldList = PatternField (',' PatternField)* (',' '..')? ','?
PatternField = IDENT (':' Pattern)?

PatternList = Pattern (',' Pattern)* ','?

RangePat = RangeBound? '..' '='? RangeBound?
RangeBound = LiteralPat | Path

RefPat = '&' 'mut'? Pattern
```

---

## 9. Statements

```
Statement = Item
          | LetStatement
          | ExprStatement

LetStatement = IDENT TypeAnnotation? '=' Expr NEWLINE
             | IDENT TypeAnnotation? ':=' Expr NEWLINE
             | Pattern '=' Expr NEWLINE

TypeAnnotation = ':' Type

ExprStatement = Expr NEWLINE
```

---

## 10. Grammar Properties

### 10.1 LL(1) Considerations

The grammar is designed to be mostly LL(1):
- Keywords are single tokens (`f`, `s`, `e`, etc.)
- Type shortcuts are unambiguous (`[T]`, `T?`, `T!`)
- Indentation provides clear block boundaries

### 10.2 Disambiguation Rules

```
# '<' is never generic in v2 (we use '[')
a < b              # always comparison

# '|' context-dependent
items | filter     # pipeline (in expression)
A | B              # or-pattern (in pattern)

# '{' disambiguation
{1, 2, 3}          # Set (elements)
{"a": 1}           # Map (key: value)
{ stmt; expr }     # Block (statements)
```

### 10.3 Error Recovery Points

```
# Statement boundary
NEWLINE at same or lesser indent

# Item boundary
'f' | 's' | 'e' | 't' | 'i' | 'md' | 'us' | '@'

# Expression boundary
')' | ']' | '}' | ','
```
