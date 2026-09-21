const MINIMUM_LENGTH = 8;
const MAXIMUM_LENGTH = 128;
const UPPERCASE = /\p{Lu}/u;
const DIGIT = /\p{Nd}/u;
const NEITHER_LETTER_NOR_DIGIT = /[^\p{L}\p{Nd}]/u;

export function isStrongPassword(password: string): boolean {
  if (password.length < MINIMUM_LENGTH || password.length > MAXIMUM_LENGTH) {
    return false;
  }

  return (
    UPPERCASE.test(password) &&
    DIGIT.test(password) &&
    NEITHER_LETTER_NOR_DIGIT.test(password)
  );
}
