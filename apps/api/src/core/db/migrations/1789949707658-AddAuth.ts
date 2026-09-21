import { MigrationInterface, QueryRunner } from "typeorm";

export class AddAuth1789949707658 implements MigrationInterface {
  name = "AddAuth1789949707658";

  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `CREATE TABLE "session" ("id" uuid NOT NULL DEFAULT gen_random_uuid(), "userId" uuid NOT NULL, "refreshTokenHash" character varying(64) NOT NULL, "expiresAt" TIMESTAMP WITH TIME ZONE NOT NULL, "ipAddress" character varying(45), "userAgent" character varying(512), "createdAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), "updatedAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), CONSTRAINT "UQ_ff3907da35cc76361715c820ca3" UNIQUE ("refreshTokenHash"), CONSTRAINT "PK_f55da76ac1c3ac420f444d2ff11" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_3d2f174ef04fb312fdebd0ddc5" ON "session"  ("userId") `,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_5d97cf9773002b16861b4bb8ae" ON "session"  ("expiresAt") `,
    );
    await queryRunner.query(
      `CREATE TABLE "user" ("id" uuid NOT NULL DEFAULT gen_random_uuid(), "name" character varying(100) NOT NULL, "email" character varying(320) NOT NULL, "emailVerified" boolean NOT NULL DEFAULT false, "image" character varying(2048), "createdAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), "updatedAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), CONSTRAINT "UQ_e12875dfb3b1d92d7d7c5377e22" UNIQUE ("email"), CONSTRAINT "PK_cace4a159ff9f2512dd42373760" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `CREATE TYPE "public"."account_provider_enum" AS ENUM('credential', 'google')`,
    );
    await queryRunner.query(
      `CREATE TABLE "account" ("id" uuid NOT NULL DEFAULT gen_random_uuid(), "userId" uuid NOT NULL, "provider" "public"."account_provider_enum" NOT NULL, "providerAccountId" character varying(255), "passwordHash" character varying(255), "createdAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), "updatedAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), CONSTRAINT "UQ_32054f2625f6b8e31319d0e020b" UNIQUE ("provider", "providerAccountId"), CONSTRAINT "UQ_8107b3776766054e4465d9f3154" UNIQUE ("userId", "provider"), CONSTRAINT "PK_54115ee388cdb6d86bb4bf5b2ea" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `CREATE TYPE "public"."verification_purpose_enum" AS ENUM('email_verification', 'password_reset')`,
    );
    await queryRunner.query(
      `CREATE TABLE "verification" ("id" uuid NOT NULL DEFAULT gen_random_uuid(), "identifier" character varying(320) NOT NULL, "purpose" "public"."verification_purpose_enum" NOT NULL, "tokenHash" character varying(64) NOT NULL, "expiresAt" TIMESTAMP WITH TIME ZONE NOT NULL, "consumedAt" TIMESTAMP WITH TIME ZONE, "createdAt" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(), CONSTRAINT "UQ_d80d0c3a64264755100b4c09729" UNIQUE ("tokenHash"), CONSTRAINT "PK_f7e3a90ca384e71d6e2e93bb340" PRIMARY KEY ("id"))`,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_b2a48be969980ae537091ef582" ON "verification"  ("expiresAt") `,
    );
    await queryRunner.query(
      `CREATE INDEX "IDX_48399d5ae1f3d05dfd4cdf64d4" ON "verification"  ("identifier", "purpose", "consumedAt") `,
    );
    await queryRunner.query(
      `ALTER TABLE "session" ADD CONSTRAINT "FK_3d2f174ef04fb312fdebd0ddc53" FOREIGN KEY ("userId") REFERENCES "user"("id") ON DELETE CASCADE ON UPDATE NO ACTION`,
    );
    await queryRunner.query(
      `ALTER TABLE "account" ADD CONSTRAINT "FK_60328bf27019ff5498c4b977421" FOREIGN KEY ("userId") REFERENCES "user"("id") ON DELETE CASCADE ON UPDATE NO ACTION`,
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(
      `ALTER TABLE "account" DROP CONSTRAINT "FK_60328bf27019ff5498c4b977421"`,
    );
    await queryRunner.query(
      `ALTER TABLE "session" DROP CONSTRAINT "FK_3d2f174ef04fb312fdebd0ddc53"`,
    );
    await queryRunner.query(
      `DROP INDEX "public"."IDX_48399d5ae1f3d05dfd4cdf64d4"`,
    );
    await queryRunner.query(
      `DROP INDEX "public"."IDX_b2a48be969980ae537091ef582"`,
    );
    await queryRunner.query(`DROP TABLE "verification"`);
    await queryRunner.query(`DROP TYPE "public"."verification_purpose_enum"`);
    await queryRunner.query(`DROP TABLE "account"`);
    await queryRunner.query(`DROP TYPE "public"."account_provider_enum"`);
    await queryRunner.query(`DROP TABLE "user"`);
    await queryRunner.query(
      `DROP INDEX "public"."IDX_5d97cf9773002b16861b4bb8ae"`,
    );
    await queryRunner.query(
      `DROP INDEX "public"."IDX_3d2f174ef04fb312fdebd0ddc5"`,
    );
    await queryRunner.query(`DROP TABLE "session"`);
  }
}
