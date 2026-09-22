import { MigrationInterface, QueryRunner } from "typeorm";

export class AddEmailDispatch1790030470517 implements MigrationInterface {
  name = "AddEmailDispatch1790030470517";

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TYPE "public"."emailDispatch_kind_enum" AS ENUM('verification', 'password_reset')`,
    );
    await queryRunner.query(
      `CREATE TABLE "emailDispatch" ("id" uuid NOT NULL DEFAULT gen_random_uuid(), "email" character varying(320) NOT NULL, "kind" "public"."emailDispatch_kind_enum" NOT NULL, "createdAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), CONSTRAINT "PK_63b82871fc971f3f3cbcad6633e" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_971884e47f07ef60a1cce2f62c" ON "emailDispatch"  ("createdAt") `,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_68e8b990aab243cb4ccea3c72a" ON "emailDispatch"  ("email", "kind", "createdAt") `,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `DROP INDEX "public"."IDX_68e8b990aab243cb4ccea3c72a"`,
    );
    await queryRunner.query(
      `DROP INDEX "public"."IDX_971884e47f07ef60a1cce2f62c"`,
    );
    await queryRunner.query(`DROP TABLE "emailDispatch"`);
    await queryRunner.query(`DROP TYPE "public"."emailDispatch_kind_enum"`);
  }
}
