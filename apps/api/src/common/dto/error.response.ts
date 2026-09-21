import { ApiProperty } from "@nestjs/swagger";
import type { ErrorCode, ErrorFields } from "../exceptions/business-error";

export class ErrorResponse {
  @ApiProperty({ example: "INVALID_SESSION" })
  code!: ErrorCode;

  @ApiProperty({ example: "Invalid session" })
  message!: string;

  @ApiProperty({
    type: "object",
    additionalProperties: { type: "string" },
    example: { password: "WEAK_PASSWORD" },
  })
  fields!: ErrorFields;
}
