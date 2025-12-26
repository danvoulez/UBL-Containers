import { Command } from "commander";
import { pushDirectory, commitRef } from "@ubl/sdk/repo";

export function mountRepo(cmd: Command) {
  const repo = cmd.command("repo").description("Repo (static container) operations");

  repo.command("push")
    .requiredOption("--tenant <tenant>")
    .requiredOption("--repo <repo>")
    .requiredOption("--ref <refName>", "e.g. refs/heads/main")
    .requiredOption("--dir <dir>", "directory to push")
    .option("--base-url <url>", "UBL server base URL", "http://localhost:8080")
    .option("--mode <mode>", "'ff' or 'force'", "ff")
    .requiredOption("--sid <sid>", "session token")
    .action(async (opts) => {
      const res = await pushDirectory(opts.baseUrl, opts.tenant, opts.repo, opts.ref, opts.dir, opts.sid, opts.mode);
      console.log(JSON.stringify(res, null, 2));
    });

  repo.command("force")
    .requiredOption("--tenant <tenant>")
    .requiredOption("--repo <repo>")
    .requiredOption("--ref <refName>")
    .requiredOption("--old <old>")
    .requiredOption("--new <new>")
    .option("--base-url <url>", "http://localhost:8080")
    .requiredOption("--sid <sid>")
    .action(async (opts) => {
      const res = await commitRef(opts.baseUrl, { tenant: opts.tenant, repo: opts.repo, ref: opts.refName, old: opts.old, new: opts.new, mode: "force" }, opts.sid);
      console.log(JSON.stringify(res, null, 2));
    });
}
