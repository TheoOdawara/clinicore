import {
  Column,
  CreateDateColumn,
  Entity,
  Index,
  PrimaryGeneratedColumn,
  Unique,
} from "typeorm";
import { VerificationPurpose } from "../enums/verification-purpose.enum";

@Entity("verification")
@Unique(["tokenHash"])
@Index(["identifier", "purpose", "consumedAt"])
export class Verification {
  @PrimaryGeneratedColumn("uuid")
  id!: string;

  @Column({ type: "varchar", length: 320 })
  identifier!: string;

  @Column({ type: "enum", enum: VerificationPurpose })
  purpose!: VerificationPurpose;

  @Column({ type: "varchar", length: 64 })
  tokenHash!: string;

  @Index()
  @Column({ type: "timestamptz" })
  expiresAt!: Date;

  @Column({ type: "timestamptz", nullable: true })
  consumedAt!: Date | null;

  @CreateDateColumn({ type: "timestamptz" })
  createdAt!: Date;
}
