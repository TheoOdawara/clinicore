import { ApiProperty } from "@nestjs/swagger";
import { IsStrongPassword } from "../utils/password-policy";
import { TokenDto } from "./token.dto";

export class PasswordResetConfirmationDto extends TokenDto {
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
