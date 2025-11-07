import React, { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  applyThemePreference,
  getStoredThemePreference,
  persistThemePreference,
  type ThemePreference,
} from '../theme';

type ThemeSwitcherProps = {
  className?: string;
};

type ThemeOption = {
  value: ThemePreference;
  icon: string;
  labelKey: string;
};

const THEME_OPTIONS: ThemeOption[] = [
  { value: 'system', icon: '🌓', labelKey: 'theme.options.system' },
  { value: 'light', icon: '🌕', labelKey: 'theme.options.light' },
  { value: 'dark', icon: '🌙', labelKey: 'theme.options.dark' },
];

const ThemeSwitcher = ({ className }: ThemeSwitcherProps): React.JSX.Element => {
  const { t } = useTranslation();
  const [preference, setPreference] = useState<ThemePreference>(() => getStoredThemePreference());

  useEffect(() => {
    persistThemePreference(preference);
    applyThemePreference(preference);
  }, [preference]);

  const handleChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const nextPreference = event.target.value as ThemePreference;
    setPreference(nextPreference);
  };

  const activeOption =
    THEME_OPTIONS.find((option) => option.value === preference) ?? THEME_OPTIONS[0];

  return (
    <label className={`theme-switcher ${className ?? ''}`}>
      <span className="sr-only">{t('theme.label')}</span>
      <span className="theme-switcher__display" aria-hidden="true">
        {activeOption.icon}
      </span>
      <select
        className="theme-switcher__select"
        value={preference}
        onChange={handleChange}
        aria-label={t('theme.label')}
      >
        {THEME_OPTIONS.map((option) => (
          <option key={option.value} value={option.value}>
            {t(option.labelKey)}
          </option>
        ))}
      </select>
    </label>
  );
};

export default ThemeSwitcher;
