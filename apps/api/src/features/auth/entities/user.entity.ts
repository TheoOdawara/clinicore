import {
  Column,
  CreateDateColumn,
  Entity,
  OneToMany,
  PrimaryGeneratedColumn,
  Unique,
  UpdateDateColumn,
} from "typeorm";
import { Account } from "./account.entity";
import { Session } from "./session.entity";

@Entity("user")
@Unique(["email"])
export class User {
  @PrimaryGeneratedColumn("uuid")
  id!: string;

  @Column({ type: "varchar", length: 100 })
  name!: string;

  @Column({ type: "varchar", length: 320 })
  email!: string;

  @Column({ type: "boolean", default: false })
  emailVerified!: boolean;

  @Column({ type: "varchar", length: 2048, nullable: true })
  image!: string | null;

  @CreateDateColumn({ type: "timestamptz" })
  createdAt!: Date;

  @UpdateDateColumn({ type: "timestamptz" })
  updatedAt!: Date;

  @OneToMany(() => Account, (account) => account.user)
  accounts!: Account[];

  @OneToMany(() => Session, (session) => session.user)
  sessions!: Session[];
}
