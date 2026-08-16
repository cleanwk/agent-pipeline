cask "agent-pipeline" do
  version "0.1.0"
  sha256 :no_check

  url "https://github.com/cleanwk/agent-pipeline/releases/download/v#{version}/Agent%20Pipeline_#{version}_aarch64.dmg",
      verified: "github.com/cleanwk/agent-pipeline/"
  name "Agent Pipeline"
  desc "Local-first Mission Control for long-running coding agent pipelines"
  homepage "https://github.com/cleanwk/agent-pipeline"

  depends_on arch: :arm64
  depends_on macos: :sonoma

  app "Agent Pipeline.app"

  zap trash: [
    "~/Library/Application Support/dev.agentpipeline.desktop",
    "~/Library/Caches/dev.agentpipeline.desktop",
    "~/Library/Preferences/dev.agentpipeline.desktop.plist",
  ]
end
