import { applyDecorators, type HttpStatus } from "@nestjs/common";
import { ApiExtraModels, ApiResponse, getSchemaPath } from "@nestjs/swagger";
import { ProblemResponse } from "../dto/problem.response";

export function ApiProblemResponse(
  status: HttpStatus,
  description: string,
): MethodDecorator & ClassDecorator {
  return applyDecorators(
    ApiExtraModels(ProblemResponse),
    ApiResponse({
      status,
      description,
      content: {
        "application/problem+json": {
          schema: { $ref: getSchemaPath(ProblemResponse) },
        },
      },
    }),
  );
}
