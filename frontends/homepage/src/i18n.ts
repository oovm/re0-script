import { computed, ref, watch } from "vue";

export type Locale = "zh-CN" | "en-US";
const storageKey = "yydb.locale";
const stored = localStorage.getItem(storageKey);

export const locale = ref<Locale>(
    stored === "zh-CN" || stored === "en-US"
        ? stored
        : navigator.language.toLowerCase().startsWith("zh")
          ? "zh-CN"
          : "en-US",
);
export const isZh = computed(() => locale.value === "zh-CN");
watch(
    locale,
    (value) => {
        localStorage.setItem(storageKey, value);
        document.documentElement.lang = value;
    },
    { immediate: true },
);
export function pick<T>(zh: T, en: T): T {
    return isZh.value ? zh : en;
}
