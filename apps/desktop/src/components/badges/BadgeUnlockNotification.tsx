import { useEffect, useState } from 'react'
import { X, Sparkles } from 'lucide-react'
import { useBadgeStore } from '@/stores/badgeStore'

export function BadgeUnlockNotification() {
  const { newlyUnlocked, clearNewlyUnlocked } = useBadgeStore()
  const [currentIndex, setCurrentIndex] = useState(0)
  const [isVisible, setIsVisible] = useState(false)
  const [isAnimating, setIsAnimating] = useState(false)

  useEffect(() => {
    if (newlyUnlocked.length > 0) {
      setCurrentIndex(0)
      setIsVisible(true)
      setIsAnimating(true)
      
      // Remove animation class after animation completes
      const timer = setTimeout(() => setIsAnimating(false), 600)
      return () => clearTimeout(timer)
    }
  }, [newlyUnlocked])

  const handleDismiss = () => {
    if (currentIndex < newlyUnlocked.length - 1) {
      // Show next badge
      setCurrentIndex(currentIndex + 1)
      setIsAnimating(true)
      setTimeout(() => setIsAnimating(false), 600)
    } else {
      // All badges shown, close notification
      setIsVisible(false)
      setTimeout(() => {
        clearNewlyUnlocked()
        setCurrentIndex(0)
      }, 300)
    }
  }

  if (!isVisible || newlyUnlocked.length === 0) {
    return null
  }

  const badge = newlyUnlocked[currentIndex]
  const hasMoreBadges = currentIndex < newlyUnlocked.length - 1

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      {/* Confetti effect placeholder - can add actual confetti library later */}
      <div className="absolute inset-0 pointer-events-none overflow-hidden">
        {[...Array(20)].map((_, i) => (
          <div
            key={i}
            className="absolute animate-confetti"
            style={{
              left: `${Math.random() * 100}%`,
              animationDelay: `${Math.random() * 2}s`,
              animationDuration: `${2 + Math.random() * 2}s`,
            }}
          >
            <Sparkles className="w-4 h-4 text-yellow-400" />
          </div>
        ))}
      </div>

      {/* Badge Card */}
      <div
        className={`
          relative bg-gradient-to-br from-yellow-50 to-orange-50 rounded-3xl 
          p-8 max-w-sm w-full mx-4 shadow-2xl border-2 border-yellow-300
          ${isAnimating ? 'animate-badge-unlock' : ''}
        `}
      >
        {/* Close button */}
        <button
          onClick={handleDismiss}
          className="absolute top-4 right-4 text-gray-400 hover:text-gray-600 transition-colors"
        >
          <X className="w-5 h-5" />
        </button>

        {/* Badge count indicator */}
        {newlyUnlocked.length > 1 && (
          <div className="absolute top-4 left-4 px-2 py-1 bg-yellow-200 rounded-full text-xs font-semibold text-yellow-800">
            {currentIndex + 1} / {newlyUnlocked.length}
          </div>
        )}

        <div className="text-center">
          {/* Celebratory header */}
          <div className="flex items-center justify-center gap-2 text-yellow-600 mb-4">
            <Sparkles className="w-5 h-5" />
            <span className="text-sm font-semibold uppercase tracking-wider">
              Badge Unlocked!
            </span>
            <Sparkles className="w-5 h-5" />
          </div>

          {/* Badge icon - large and animated */}
          <div
            className={`
              text-7xl mb-4 
              ${isAnimating ? 'animate-bounce-once' : ''}
            `}
          >
            {badge.icon}
          </div>

          {/* Badge name */}
          <h2 className="text-2xl font-bold text-gray-800 mb-2">{badge.name}</h2>

          {/* Badge description */}
          <p className="text-gray-600 mb-6">{badge.description}</p>

          {/* Action button */}
          <button
            onClick={handleDismiss}
            className="w-full py-3 bg-gradient-to-r from-yellow-400 to-orange-400 
                     hover:from-yellow-500 hover:to-orange-500
                     text-white font-semibold rounded-xl shadow-lg 
                     hover:shadow-xl transition-all transform hover:scale-[1.02]"
          >
            {hasMoreBadges ? 'Next Badge →' : 'Awesome!'}
          </button>
        </div>
      </div>

      {/* CSS for animations */}
      <style>{`
        @keyframes badge-unlock {
          0% {
            transform: scale(0.5);
            opacity: 0;
          }
          50% {
            transform: scale(1.05);
          }
          100% {
            transform: scale(1);
            opacity: 1;
          }
        }
        
        @keyframes bounce-once {
          0%, 100% {
            transform: translateY(0);
          }
          50% {
            transform: translateY(-20px);
          }
        }
        
        @keyframes confetti {
          0% {
            transform: translateY(-100vh) rotate(0deg);
            opacity: 1;
          }
          100% {
            transform: translateY(100vh) rotate(720deg);
            opacity: 0;
          }
        }
        
        .animate-badge-unlock {
          animation: badge-unlock 0.5s ease-out forwards;
        }
        
        .animate-bounce-once {
          animation: bounce-once 0.6s ease-out;
        }
        
        .animate-confetti {
          animation: confetti 3s ease-in-out infinite;
        }
      `}</style>
    </div>
  )
}
