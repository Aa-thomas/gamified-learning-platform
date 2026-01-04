import { Link, useLocation } from 'react-router-dom'
import { Home, Map, BarChart3, Settings, Trophy, Brain, BookOpen } from 'lucide-react'
import { useReviewStore } from '@/stores/reviewStore'
import { useCurriculumStore } from '@/stores/curriculumStore'
import { useEffect } from 'react'

const navItems = [
  { path: '/', label: 'Home', icon: Home },
  { path: '/skill-tree', label: 'Skill Tree', icon: Map },
  { path: '/badges', label: 'Badges', icon: Trophy },
  { path: '/review', label: 'Review', icon: Brain, showBadge: true },
  { path: '/progress', label: 'Progress', icon: BarChart3 },
  { path: '/curriculum', label: 'Curriculum', icon: BookOpen },
  { path: '/settings', label: 'Settings', icon: Settings },
]

export function Navigation() {
  const location = useLocation()
  const { dueCount, fetchDueCount } = useReviewStore()
  const { activeCurriculum, fetchActiveCurriculum } = useCurriculumStore()

  useEffect(() => {
    fetchDueCount()
    fetchActiveCurriculum()
  }, [fetchDueCount, fetchActiveCurriculum])

  return (
    <nav className="bg-white border-b border-gray-200 px-4 py-2">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-xl font-bold text-primary">🦀 RustCamp</span>
          {activeCurriculum && (
            <span className="text-xs bg-gray-100 text-gray-600 px-2 py-1 rounded-full">
              {activeCurriculum.name}
            </span>
          )}
        </div>

        <div className="flex items-center gap-1">
          {navItems.map(({ path, label, icon: Icon, showBadge }) => {
            const isActive = location.pathname === path
            const showReviewBadge = showBadge && dueCount > 0
            return (
              <Link
                key={path}
                to={path}
                className={`
                  relative flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium
                  transition-colors
                  ${isActive
                    ? 'bg-primary text-white'
                    : 'text-gray-600 hover:bg-gray-100'
                  }
                `}
              >
                <Icon size={18} />
                <span className="hidden sm:inline">{label}</span>
                {showReviewBadge && (
                  <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs 
                                 w-5 h-5 rounded-full flex items-center justify-center font-bold">
                    {dueCount > 9 ? '9+' : dueCount}
                  </span>
                )}
              </Link>
            )
          })}
        </div>
      </div>
    </nav>
  )
}
