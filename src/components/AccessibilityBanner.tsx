import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { isMac } from "@/lib/platform";

/**
 * On a Mac, a strip under the titlebar while Talk lacks the Accessibility permission. Without
 * it the paste is refused and a modifier held alone never reaches Talk, both without a word,
 * so a rebuilt or renamed application that lost the permission looked simply broken. Read
 * again whenever the window comes back to the front, which is where the user returns from
 * System Settings.
 */
export function AccessibilityBanner() {
  const { t } = useTranslation();
  const [granted, setGranted] = useState(true);

  useEffect(() => {
    if (!isMac) return;
    const read = () =>
      invoke<boolean>("accessibility_granted")
        .then(setGranted)
        .catch((error) => console.error("Failed to read the Accessibility permission:", error));
    read();
    window.addEventListener("focus", read);
    return () => window.removeEventListener("focus", read);
  }, []);

  if (!isMac || granted) return null;

  return (
    <div role="alert" className="shrink-0 flex items-center gap-3 border-b border-border-subtle bg-surface-raised px-4 py-2.5">
      <ShieldAlert className="h-4 w-4 shrink-0 text-warning" aria-hidden="true" />
      <p className="min-w-0 flex-1 text-sm">
        <span className="font-medium">{t("accessibilityBanner.title")}</span>
        <span className="ml-2 text-muted-foreground">{t("accessibilityBanner.why")}</span>
      </p>
      <Button size="sm" variant="outline" onClick={() => void invoke("open_accessibility_settings")}>
        {t("accessibilityBanner.open")}
      </Button>
    </div>
  );
}
