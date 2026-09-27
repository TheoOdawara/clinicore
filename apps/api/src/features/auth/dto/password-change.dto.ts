import { ApiProperty } from "@nestjs/swagger";
import { IsString, Length } from "class-validator";
import { IsStrongPassword } from "../utils/password-policy";

export class PasswordChangeDto {
  @IsString()
  @Length(1, 128)
  currentPassword!: string;

  @ApiProperty({
    minLength: 8,
    maxLength: 128,
    description:
      "At least 8 and at most 128 characters, with an uppercase letter, a digit and a character that is neither a letter nor a digit",
    example: "Outra#Senha9",
  })
  @IsStrongPassword()
  newPassword!: string;
}
