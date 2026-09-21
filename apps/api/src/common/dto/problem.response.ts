import { ApiProperty, ApiPropertyOptional } from "@nestjs/swagger";

class FieldErrorResponse {
  @ApiProperty({ example: "#/password" })
  pointer!: string;

  @ApiProperty({ example: "WEAK_PASSWORD" })
  code!: string;
}

export class ProblemResponse {
  @ApiProperty({ example: "tag:clinicore.com.br,2026:invalid-session" })
  type!: string;

  @ApiProperty({ example: "Invalid session" })
  title!: string;

  @ApiProperty({ example: 401 })
  status!: number;

  @ApiPropertyOptional({ type: [FieldErrorResponse] })
  errors?: FieldErrorResponse[];
}
