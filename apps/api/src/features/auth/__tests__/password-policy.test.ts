import { isStrongPassword } from "../utils/password-policy";

describe("isStrongPassword", () => {
  it("accepts a password that meets every condition", () => {
    expect(isStrongPassword("Clinica#2026")).toBe(true);
  });

  it("refuses a password without an uppercase letter", () => {
    expect(isStrongPassword("sem_maiuscula#1")).toBe(false);
  });

  it("refuses a password without a digit", () => {
    expect(isStrongPassword("SEM_DIGITO#a")).toBe(false);
  });

  it("refuses a password without a special character", () => {
    expect(isStrongPassword("SemEspecial1")).toBe(false);
  });

  it("refuses a password shorter than 8 characters", () => {
    expect(isStrongPassword("Aa#1")).toBe(false);
    expect(isStrongPassword("Abc#123")).toBe(false);
    expect(isStrongPassword("Abcd#123")).toBe(true);
  });

  it("refuses a password longer than 128 characters", () => {
    const filler = "a".repeat(120);
    expect(isStrongPassword(`Abcd#123${filler}`)).toBe(true);
    expect(isStrongPassword(`Abcd#123${filler}a`)).toBe(false);
  });

  it("counts a space and an accented letter for what they are", () => {
    expect(isStrongPassword("Clinica 2026")).toBe(true);
    expect(isStrongPassword("Clinicaá2026")).toBe(false);
  });
});
