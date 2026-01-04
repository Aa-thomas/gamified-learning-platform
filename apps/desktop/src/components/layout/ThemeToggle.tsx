import { Sun, Moon, Monitor } from 'lucide-react'
import { useThemeStore, Theme } from '@/stores/themeStore'

const icons: Record<Theme, typeof Sun> = {
  light: Sun,
  dark: Moon,
  system: Monitor,
}

const labels: Record<Theme, string> = {
  light: 'Light mode',
  dark: 'Dark mode',
  system: 'System theme',
}

export function ThemeToggle() {
  const { theme, toggleTheme } = useThemeStore()
  const Icon = icons[theme]

  return (
    <button
      onClick={toggleTheme}
      className="p-2 rounded-lg text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
      title={labels[theme]}
      aria-label={`Current theme: ${labels[theme]}. Click to change.`}
    >
      <Icon className="w-5 h-5" />
    </button>
  )
}
