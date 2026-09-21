import { SetMetadata, type CustomDecorator } from "@nestjs/common";

export const IS_PUBLIC = "isPublic";

export function Public(): CustomDecorator {
  return SetMetadata(IS_PUBLIC, true);
}
