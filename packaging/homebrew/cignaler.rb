cask "cignaler" do
  version "0.0.2"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"

  url "https://github.com/cignaler/cignaler/releases/download/v#{version}/cignaler_#{version}_universal.dmg"
  name "Cignaler"
  desc "Watches GitLab CI/CD pipelines from the system tray"
  homepage "https://github.com/cignaler/cignaler"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :catalina"

  app "cignaler.app"

  zap trash: [
    "~/Library/Application Support/com.ostwi.dev",
    "~/Library/Application Support/Google/Chrome/NativeMessagingHosts/com.cignaler.app.json",
    "~/Library/Caches/com.ostwi.dev",
    "~/Library/HTTPStorages/com.ostwi.dev",
    "~/Library/Saved Application State/com.ostwi.dev.savedState",
    "~/Library/WebKit/com.ostwi.dev",
  ]
end
