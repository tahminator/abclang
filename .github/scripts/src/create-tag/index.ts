import {
  GitHubClient,
  Utils,
  VersioningClient,
  VersionUpdatingStrategy,
} from "@tahminator/pipeline";

const { required } = Utils;

export async function main() {
  const { githubAppAppId, githubAppInstallationId, githubAppPrivateKey } =
    parseCiEnv();

  const ghClient = await GitHubClient.createWithGithubAppToken({
    appId: githubAppAppId,
    installationId: githubAppInstallationId,
    privateKey: githubAppPrivateKey,
  });

  const versioningClient = new VersioningClient(
    ghClient,
    VersionUpdatingStrategy.JSTS,
  );

  await ghClient.createTag({
    nextTag: await versioningClient.next(),
  });
}

function parseCiEnv() {
  const githubAppAppId = required(process.env["_GITHUB_APP_APP_ID"]);

  const githubAppInstallationId = required(
    process.env["_GITHUB_APP_INSTALLATION_ID"],
  );

  const githubAppPrivateKey = required(process.env["_GITHUB_APP_PEM_CONTENT"]);

  return { githubAppAppId, githubAppInstallationId, githubAppPrivateKey };
}

main()
  .then(() => {
    process.exit(0);
  })
  .catch((e) => {
    console.error(e);
    process.exit(1);
  });
