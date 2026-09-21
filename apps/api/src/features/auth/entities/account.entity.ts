import {
  Column,
  CreateDateColumn,
  Entity,
  JoinColumn,
  ManyToOne,
  PrimaryGeneratedColumn,
  Unique,
  UpdateDateColumn,
} from "typeorm";
import { Provider } from "../enums/provider.enum";
import { User } from "./user.entity";

@Entity("account")
@Unique(["userId", "provider"])
@Unique(["provider", "providerAccountId"])
export class Account {
  @PrimaryGeneratedColumn("uuid")
  id!: string;

  @Column({ type: "uuid" })
  userId!: string;

  @Column({ type: "enum", enum: Provider })
  provider!: Provider;

  @Column({ type: "varchar", length: 255, nullable: true })
  providerAccountId!: string | null;

  @Column({ type: "varchar", length: 255, nullable: true })
  passwordHash!: string | null;

  @CreateDateColumn({ type: "timestamptz" })
  createdAt!: Date;

  @UpdateDateColumn({ type: "timestamptz" })
  updatedAt!: Date;

  @ManyToOne(() => User, (user) => user.accounts, { onDelete: "CASCADE" })
  @JoinColumn({ name: "userId" })
  user!: User;
}
