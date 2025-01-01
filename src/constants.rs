pub static KEYWORDS: &'static [&'static str] = &[
  "abstract type",
  "end",
  "function"
];

pub static OPERATORS: &'static [char] = &[
  // function call stuff
  '(', ')',
  '{', '}',
  ',',
  // binary operations
  '+',
  '-',
  '*',
  '/',
  // assignment
  '='
];

pub static OTHER_OPERATORS: &'static [char] = &[
  // function call stuff
  '(', ')',
  '{', '}',
  ','
];

pub static BINARY_OPERATORS: &'static [char] = &[
  // binary operations
  '+',
  '-',
  '*',
  '/'
];