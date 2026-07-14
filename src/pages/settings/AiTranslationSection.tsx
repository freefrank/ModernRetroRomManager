import { useEffect, useState } from "react";
import { Bot, KeyRound } from "lucide-react";
import { useTranslation } from "react-i18next";
import { aiTranslationApi } from "@/lib/api";
import { Badge, Button, Input, toast } from "@/components/ui";

export default function AiTranslationSection() {
  const { t } = useTranslation();
  const [endpoint, setEndpoint] = useState("https://api.openai.com/v1");
  const [model, setModel] = useState("");
  const [targetLanguage, setTargetLanguage] = useState("简体中文（zh-CN）");
  const [apiKey, setApiKey] = useState("");
  const [hasApiKey, setHasApiKey] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    aiTranslationApi.getConfig().then(config => {
      setEndpoint(config.endpoint);
      setModel(config.model);
      setTargetLanguage(config.target_language);
      setHasApiKey(config.has_api_key);
    }).catch(error => toast.error(t("settings.aiTranslation.loadFailed", { error: String(error) })));
  }, [t]);

  const save = async () => {
    setSaving(true);
    try {
      await aiTranslationApi.saveConfig({
        endpoint,
        model,
        target_language: targetLanguage,
        api_key: apiKey || undefined,
      });
      if (apiKey) setHasApiKey(true);
      setApiKey("");
      toast.success(t("settings.aiTranslation.saved"));
    } catch (error) {
      toast.error(t("settings.aiTranslation.saveFailed", { error: String(error) }));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <section>
        <div className="flex items-center gap-3 mb-2">
          <Bot className="w-5 h-5 text-accent-primary" />
          <h2 className="text-lg font-medium text-text-primary">{t("settings.aiTranslation.title")}</h2>
          {hasApiKey && <Badge variant="info">{t("settings.aiTranslation.configured")}</Badge>}
        </div>
        <p className="text-sm text-text-secondary">{t("settings.aiTranslation.description")}</p>
      </section>

      <section className="space-y-4 rounded-[var(--radius-lg)] border border-border-default bg-bg-secondary p-4">
        <label className="block space-y-1.5 text-sm text-text-primary">
          <span>{t("settings.aiTranslation.endpoint")}</span>
          <Input value={endpoint} onChange={event => setEndpoint(event.target.value)} placeholder="https://api.example.com/v1" />
        </label>
        <label className="block space-y-1.5 text-sm text-text-primary">
          <span>{t("settings.aiTranslation.apiKey")}</span>
          <div className="relative">
            <KeyRound className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-muted" />
            <Input type="password" value={apiKey} onChange={event => setApiKey(event.target.value)} className="pl-9" placeholder={hasApiKey ? t("settings.aiTranslation.savedKey") : "sk-..."} />
          </div>
        </label>
        <label className="block space-y-1.5 text-sm text-text-primary">
          <span>{t("settings.aiTranslation.model")}</span>
          <Input value={model} onChange={event => setModel(event.target.value)} placeholder="gpt-4o-mini" />
        </label>
        <label className="block space-y-1.5 text-sm text-text-primary">
          <span>{t("settings.aiTranslation.targetLanguage")}</span>
          <Input value={targetLanguage} onChange={event => setTargetLanguage(event.target.value)} />
        </label>
        <p className="text-xs leading-relaxed text-text-muted">{t("settings.aiTranslation.securityHint")}</p>
        <Button onClick={save} loading={saving} disabled={!endpoint.trim() || !model.trim() || !targetLanguage.trim()}>
          {t("common.save")}
        </Button>
      </section>
    </div>
  );
}
