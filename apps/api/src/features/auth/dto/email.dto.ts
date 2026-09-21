import { IsEmail, Length } from "class-validator";

export class EmailDto {
  @IsEmail()
  @Length(1, 320)
  email!: string;
}
