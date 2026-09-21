import { IsEmail, IsString, Length } from "class-validator";

export class SignInDto {
  @IsEmail()
  @Length(1, 320)
  email!: string;

  @IsString()
  @Length(1, 128)
  password!: string;
}
