import { Command } from "commander";
import { mountRepo } from "./commands/repo";

const program = new Command();
program.name("ubl").description("UBL CLI");
mountRepo(program);

program.parseAsync(process.argv).catch(e => {
  console.error(e);
  process.exit(1);
});
