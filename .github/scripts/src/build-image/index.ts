import { DockerClient, GitHubClient } from "@tahminator/pipeline";
import { $ } from "bun";
import yargs from "yargs";
import { hideBin } from "yargs/helpers";

const { dockerUpload, getGhaOutput, githubOutputFile } = await yargs(
  hideBin(process.argv),
)
  .option("dockerUpload", {
    type: "boolean",
    default: true,
  })
  .option("getGhaOutput", {
    type: "boolean",
    describe:
      "Enable GitHub Actions output to receive latest built tag version",
    default: false,
  })
  .option("githubOutputFile", {
    type: "string",
    describe:
      "Path to GITHUB_OUTPUT (this will be passed in automatically in CI)",
    default: process.env.GITHUB_OUTPUT,
  })
  .strict()
  .parse();

async function main() {
  const { dockerHubPat } = parseCiEnv(process.env);

  const ghClient = await GitHubClient.createWithDefaultCiToken();
  await using dockerClient = await DockerClient.create(
    "tahminator",
    dockerHubPat,
  );

  const timestamp = new Date().toISOString();

  const gitSha = (await $`git rev-parse --short HEAD`.text()).trim();

  await dockerClient.buildImage({
    dockerRepository: "abclang",
    dockerFileLocation: `infra/Dockerfile`,
    tags: [`${timestamp}`, `${gitSha}`],
    shouldUpload: dockerUpload,
  });

  if (getGhaOutput) {
    await ghClient.outputToGithubOutput({
      overrideGithubOutputFile: githubOutputFile ? githubOutputFile : undefined,
      ctx: {
        tag: `${gitSha}`,
      },
    });
  }
}

function parseCiEnv(ciEnv: Record<string, string | undefined>) {
  const dockerHubPat = (() => {
    const v = ciEnv["DOCKER_HUB_PAT"];
    if (!v) {
      throw new Error("Missing DOCKER_HUB_PAT from env");
    }
    return v;
  })();

  return { dockerHubPat };
}

main()
  .then(() => {
    process.exit(0);
  })
  .catch((e) => {
    console.error(e);
    process.exit(1);
  });
