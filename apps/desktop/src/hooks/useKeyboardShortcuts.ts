import { useEffect, useCallback, useState } from 'react'
import { useNavigate } from 'react-router-dom'

export interface Shortcut {
  key: string
  ctrl?: boolean
  shift?: boolean
  alt?: boolean
  description: string
  action: () => void
}

export interface ShortcutGroup {
  name: string
  shortcuts: Shortcut[]
}

export function useKeyboardShortcuts() {
  const navigate = useNavigate()
  const [isHelpModalOpen, setIsHelpModalOpen] = useState(false)

  const openHelpModal = useCallback(() => setIsHelpModalOpen(true), [])
  const closeHelpModal = useCallback(() => setIsHelpModalOpen(false), [])

  const shortcuts: ShortcutGroup[] = [
    {
      name: 'Navigation',
      shortcuts: [
        { key: '1', ctrl: true, description: 'Go to Dashboard', action: () => navigate('/') },
        { key: '2', ctrl: true, description: 'Go to Skill Tree', action: () => navigate('/skill-tree') },
        { key: '3', ctrl: true, description: 'Go to Progress', action: () => navigate('/progress') },
        { key: '4', ctrl: true, description: 'Go to Review', action: () => navigate('/review') },
        { key: ',', ctrl: true, description: 'Open Settings', action: () => navigate('/settings') },
      ],
    },
    {
      name: 'General',
      shortcuts: [
        { key: '/', ctrl: true, description: 'Show keyboard shortcuts', action: openHelpModal },
        { key: 'Escape', description: 'Close modal / Cancel', action: () => {} },
      ],
    },
  ]

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Don't trigger shortcuts when typing in inputs
      const target = event.target as HTMLElement
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return
      }

      for (const group of shortcuts) {
        for (const shortcut of group.shortcuts) {
          const ctrlMatch = shortcut.ctrl ? (event.ctrlKey || event.metaKey) : !event.ctrlKey && !event.metaKey
          const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey
          const altMatch = shortcut.alt ? event.altKey : !event.altKey
          const keyMatch = event.key.toLowerCase() === shortcut.key.toLowerCase()

          if (keyMatch && ctrlMatch && shiftMatch && altMatch) {
            event.preventDefault()
            shortcut.action()
            return
          }
        }
      }
    },
    [shortcuts]
  )

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  return {
    shortcuts,
    isHelpModalOpen,
    openHelpModal,
    closeHelpModal,
  }
}

export function formatShortcut(shortcut: Shortcut): string {
  const parts: string[] = []
  if (shortcut.ctrl) parts.push('Ctrl')
  if (shortcut.shift) parts.push('Shift')
  if (shortcut.alt) parts.push('Alt')
  parts.push(shortcut.key === ' ' ? 'Space' : shortcut.key.toUpperCase())
  return parts.join(' + ')
}
