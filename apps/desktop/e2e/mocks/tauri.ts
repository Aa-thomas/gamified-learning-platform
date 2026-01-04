import { Page } from '@playwright/test'

/**
 * Mock state for simulating Tauri backend behavior.
 * This allows tests to control what the "backend" returns.
 */
export interface MockState {
  user: MockUser | null
  progress: Record<string, MockNodeProgress>
  masteryScores: Record<string, number>
  contentTree: MockContentTree | null
  badges: MockBadge[]
}

export interface MockUser {
  id: string | number
  username: string
  xp: number
  level: number
  streak_days: number
  last_active_date: string
  created_at: string
}

export interface MockNodeProgress {
  id: number
  user_id: number
  node_id: string
  status: 'locked' | 'available' | 'in_progress' | 'completed'
  started_at?: string
  completed_at?: string
  xp_earned: number
  attempts: number
}

export interface MockContentTree {
  weeks: MockWeek[]
}

export interface MockWeek {
  id: string
  title: string
  days: MockDay[]
}

export interface MockDay {
  id: string
  title: string
  nodes: MockNode[]
}

export interface MockNode {
  id: string
  title: string
  node_type: 'lecture' | 'quiz' | 'challenge'
  description: string
  difficulty: string
  xp_reward: number
}

export interface MockBadge {
  id: string
  title: string
  description: string
  icon: string
  unlocked: boolean
  unlocked_at?: string
}

export interface MockQuizResult {
  score: number
  total: number
  score_percentage: number
  passed: boolean
  xp_earned: number
  attempt_number: number
  mastery_updates: Record<string, number>
  feedback: Array<{
    question_id: string
    user_answer: string | null
    correct_answer: string
    is_correct: boolean
    explanation: string
  }>
}

/**
 * Create initial mock state with sensible defaults.
 */
export function createMockState(overrides: Partial<MockState> = {}): MockState {
  return {
    user: null,
    progress: {},
    masteryScores: {},
    contentTree: null,
    badges: [],
    ...overrides,
  }
}

/**
 * XP calculation helpers (mirrors Rust backend formulas).
 */
export const formulas = {
  calculateLevel(totalXp: number): number {
    if (totalXp < 0) return 1
    let level = 1
    while (this.xpRequiredForLevel(level + 1) <= totalXp) {
      level++
    }
    return level
  },

  xpRequiredForLevel(level: number): number {
    if (level <= 1) return 0
    return Math.round(100 * Math.pow(level, 1.5))
  },

  xpToNextLevel(currentXp: number): { progress: number; total: number } {
    const currentLevel = this.calculateLevel(currentXp)
    const nextLevelXp = this.xpRequiredForLevel(currentLevel + 1)
    const currentLevelXp = this.xpRequiredForLevel(currentLevel)
    return {
      progress: currentXp - currentLevelXp,
      total: nextLevelXp - currentLevelXp,
    }
  },

  getStreakMultiplier(streakDays: number): number {
    if (streakDays <= 3) return 1.0
    if (streakDays <= 7) return 1.1
    if (streakDays <= 14) return 1.2
    if (streakDays <= 30) return 1.3
    return 1.5
  },

  getAccuracyMultiplier(accuracy: number): number {
    if (accuracy >= 100) return 1.5
    if (accuracy >= 90) return 1.3
    if (accuracy >= 80) return 1.1
    if (accuracy >= 70) return 1.0
    if (accuracy >= 60) return 0.8
    return 0.5
  },

  getRetakeMultiplier(attempt: number): number {
    if (attempt <= 0) return 0
    if (attempt === 1) return 1.0
    if (attempt === 2) return 0.5
    if (attempt === 3) return 0.25
    return 0.1
  },
}

/**
 * Create command handlers that simulate Tauri backend behavior.
 */
function createCommandHandlers(state: MockState) {
  return {
    get_user_data: () => state.user,

    create_user: (args: { username: string }) => {
      const now = new Date().toISOString()
      state.user = {
        id: `user-${Date.now()}`,
        username: args.username,
        xp: 0,
        level: 1,
        streak_days: 0,
        last_active_date: now,
        created_at: now,
        xp_for_next_level: formulas.xpRequiredForLevel(2),
        xp_progress_percentage: 0,
      }
      return state.user
    },

    update_user_xp: (args: { xpDelta: number }) => {
      if (!state.user) throw new Error('No user logged in')
      state.user.xp += args.xpDelta
      state.user.level = formulas.calculateLevel(state.user.xp)
      const { progress, total } = formulas.xpToNextLevel(state.user.xp)
      state.user.xp_for_next_level = formulas.xpRequiredForLevel(state.user.level + 1)
      state.user.xp_progress_percentage = total > 0 ? (progress / total) * 100 : 0
      return state.user
    },

    get_all_progress: () => Object.values(state.progress),

    get_node_progress: (args: { nodeId: string }) => state.progress[args.nodeId] || null,

    start_node: (args: { nodeId: string }) => {
      const now = new Date().toISOString()
      state.progress[args.nodeId] = {
        id: Object.keys(state.progress).length + 1,
        user_id: 1,
        node_id: args.nodeId,
        status: 'in_progress',
        started_at: now,
        xp_earned: 0,
        attempts: 0,
      }
      return state.progress[args.nodeId]
    },

    mark_node_complete: (args: { nodeId: string; xpEarned?: number }) => {
      const now = new Date().toISOString()
      const existing = state.progress[args.nodeId]
      state.progress[args.nodeId] = {
        ...(existing || {
          id: Object.keys(state.progress).length + 1,
          user_id: 1,
          node_id: args.nodeId,
          started_at: now,
          attempts: 0,
        }),
        status: 'completed',
        completed_at: now,
        xp_earned: args.xpEarned || 0,
      }
      return state.progress[args.nodeId]
    },

    get_content_tree: () => state.contentTree,

    get_badges: () => state.badges,

    check_badge_unlocks: () => {
      // Return any newly unlocked badges
      return state.badges.filter((b) => b.unlocked)
    },

    submit_quiz: (args: { quizId: string; answers: Record<string, string> }): MockQuizResult => {
      // Simulate quiz grading
      const correctAnswers = 2 // Assume 2/3 correct for testing
      const totalQuestions = 3
      const scorePercentage = (correctAnswers / totalQuestions) * 100
      
      const attemptNumber = (state.progress[args.quizId]?.attempts || 0) + 1
      const retakeMultiplier = formulas.getRetakeMultiplier(attemptNumber)
      const accuracyMultiplier = formulas.getAccuracyMultiplier(scorePercentage)
      const streakMultiplier = formulas.getStreakMultiplier(state.user?.streak_days || 0)
      
      const baseXp = 50 // QUIZ_BASE_XP
      const xpEarned = Math.round(baseXp * accuracyMultiplier * streakMultiplier * retakeMultiplier)

      // Update progress
      state.progress[args.quizId] = {
        id: Object.keys(state.progress).length + 1,
        user_id: 1,
        node_id: args.quizId,
        status: scorePercentage >= 70 ? 'completed' : 'in_progress',
        started_at: new Date().toISOString(),
        completed_at: scorePercentage >= 70 ? new Date().toISOString() : undefined,
        xp_earned: xpEarned,
        attempts: attemptNumber,
      }

      return {
        score: correctAnswers,
        total: totalQuestions,
        score_percentage: scorePercentage,
        passed: scorePercentage >= 70,
        xp_earned: xpEarned,
        attempt_number: attemptNumber,
        mastery_updates: { rust: 0.2 },
        feedback: [
          {
            question_id: 'q1',
            user_answer: args.answers['q1'] || null,
            correct_answer: 'b',
            is_correct: args.answers['q1'] === 'b',
            explanation: 'Correct answer explanation',
          },
        ],
      }
    },

    get_mastery_scores: () => Object.entries(state.masteryScores).map(([skill_id, score]) => ({
      skill_id,
      score,
    })),

    // System commands
    check_docker_status: () => ({ installed: true, running: true }),
    check_api_key: () => ({ configured: true, valid: true }),
    get_system_info: () => ({
      os: 'linux',
      arch: 'x86_64',
      docker_available: true,
    }),
  }
}

/**
 * Inject Tauri mock into the page context.
 * This must be called before navigating to any page.
 */
export async function injectTauriMock(page: Page, state: MockState): Promise<void> {
  const handlers = createCommandHandlers(state)

  await page.addInitScript((handlersCode) => {
    // Parse the serialized state/handlers
    const handlers = eval(`(${handlersCode})`)

    // Create mock state that persists across invocations
    const mockState = handlers.__state

    // Mock @tauri-apps/api/core invoke function
    ;(window as any).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string, args: any = {}) => {
        console.log(`[Tauri Mock] invoke("${cmd}")`, args)

        const handler = handlers[cmd]
        if (handler) {
          try {
            const result = handler(args)
            console.log(`[Tauri Mock] ${cmd} result:`, result)
            return result
          } catch (error) {
            console.error(`[Tauri Mock] ${cmd} error:`, error)
            throw error
          }
        }

        console.warn(`[Tauri Mock] Unknown command: ${cmd}`)
        throw new Error(`Unknown command: ${cmd}`)
      },
    }

    console.log('[Tauri Mock] Injected successfully')
  }, serializeHandlers(handlers, state))
}

/**
 * Serialize handlers and state for injection into browser context.
 */
function serializeHandlers(handlers: ReturnType<typeof createCommandHandlers>, state: MockState): string {
  // Use an IIFE to create a closure that captures the state
  const stateJson = JSON.stringify(state)
  
  return `(function() {
    var state = ${stateJson};
    
    return {
      get_user_data: function() { return state.user },
      
      create_user: function(args) {
        var now = new Date().toISOString();
        state.user = {
          id: Date.now(),
          username: args.username,
          xp: 0,
          level: 1,
          streak_days: 0,
          last_active_date: now,
          created_at: now,
        };
        return state.user;
      },
      
      update_user_xp: function(args) {
        if (!state.user) throw new Error('No user logged in');
        state.user.xp += args.xpDelta;
        var level = 1;
        var xp = state.user.xp;
        while (Math.round(100 * Math.pow(level + 1, 1.5)) <= xp) level++;
        state.user.level = level;
        return state.user;
      },
      
      get_all_progress: function() { return Object.values(state.progress); },
      
      get_node_progress: function(args) { return state.progress[args.nodeId] || null; },
      
      start_node: function(args) {
        var now = new Date().toISOString();
        state.progress[args.nodeId] = {
          id: Object.keys(state.progress).length + 1,
          user_id: 1,
          node_id: args.nodeId,
          status: 'in_progress',
          started_at: now,
          xp_earned: 0,
          attempts: 0,
        };
        return state.progress[args.nodeId];
      },
      
      mark_node_complete: function(args) {
        var now = new Date().toISOString();
        var existing = state.progress[args.nodeId] || {};
        state.progress[args.nodeId] = {
          id: existing.id || Object.keys(state.progress).length + 1,
          user_id: 1,
          node_id: args.nodeId,
          status: 'completed',
          started_at: existing.started_at || now,
          completed_at: now,
          xp_earned: args.xpEarned || 0,
          attempts: existing.attempts || 0,
        };
        return state.progress[args.nodeId];
      },
      
      get_content_tree: function() { return state.contentTree; },
      
      get_node_by_id: function(args) {
        if (!state.contentTree) return null;
        for (var w = 0; w < state.contentTree.weeks.length; w++) {
          var week = state.contentTree.weeks[w];
          for (var d = 0; d < week.days.length; d++) {
            var day = week.days[d];
            for (var n = 0; n < day.nodes.length; n++) {
              if (day.nodes[n].id === args.nodeId) return day.nodes[n];
            }
          }
        }
        return null;
      },
      
      get_badges: function() { return state.badges; },
      
      check_badge_unlocks: function() { return state.badges.filter(function(b) { return b.unlocked; }); },
      
      load_quiz: function(args) {
        return {
          id: args.quizId,
          title: 'Sample Quiz',
          description: 'A test quiz',
          difficulty: 'Easy',
          passing_score: 70,
          questions: [
            {
              id: 'q1',
              prompt: 'What is 2+2?',
              options: [
                { id: 'a', text: '3' },
                { id: 'b', text: '4' },
                { id: 'c', text: '5' },
              ],
              points: 10,
            },
            {
              id: 'q2',
              prompt: 'What is ownership in Rust?',
              options: [
                { id: 'a', text: 'Memory management' },
                { id: 'b', text: 'A design pattern' },
              ],
              points: 10,
            },
          ],
        };
      },
      
      submit_quiz: function(args) {
        var request = args.request || args;
        var quizId = request.quiz_id || request.quizId;
        var answers = request.answers || {};
        var correctAnswers = 2;
        var totalQuestions = 2;
        var scorePercentage = (correctAnswers / totalQuestions) * 100;
        var attemptNumber = (state.progress[quizId] ? state.progress[quizId].attempts : 0) + 1;
        var xpEarned = Math.round(50 * 1.1 * 1.0 * (attemptNumber === 1 ? 1.0 : 0.5));
        
        state.progress[quizId] = {
          id: Object.keys(state.progress).length + 1,
          user_id: 1,
          node_id: quizId,
          status: scorePercentage >= 70 ? 'completed' : 'in_progress',
          started_at: new Date().toISOString(),
          completed_at: scorePercentage >= 70 ? new Date().toISOString() : undefined,
          xp_earned: xpEarned,
          attempts: attemptNumber,
        };
        
        return {
          score: correctAnswers * 10,
          total: totalQuestions * 10,
          score_percentage: scorePercentage,
          passed: scorePercentage >= 70,
          xp_earned: xpEarned,
          attempt_number: attemptNumber,
          mastery_updates: { rust: 0.2 },
          feedback: [
            {
              question_id: 'q1',
              user_answer: answers.q1 || null,
              correct_answer: 'b',
              is_correct: answers.q1 === 'b',
              explanation: 'Basic math',
            },
            {
              question_id: 'q2',
              user_answer: answers.q2 || null,
              correct_answer: 'a',
              is_correct: answers.q2 === 'a',
              explanation: 'Memory management system',
            },
          ],
        };
      },
      
      get_mastery_scores: function() {
        return Object.entries(state.masteryScores).map(function(entry) {
          return { skill_id: entry[0], score: entry[1] };
        });
      },
      
      get_due_count: function() { return 0; },
      
      get_due_review_count: function() { return 0; },
      
      get_due_reviews: function() { return []; },
      
      get_active_curriculum: function() {
        return {
          id: 'test',
          name: 'Test Curriculum',
          version: '1.0.0',
          description: 'A test curriculum',
          author: 'Test Author',
          imported_at: new Date().toISOString(),
          is_active: true,
          stats: null,
        };
      },
      
      list_curricula: function() { return []; },
      
      check_docker_status: function() { return { installed: true, running: true }; },
      check_api_key: function() { return { configured: true, valid: true }; },
      get_system_info: function() { return { os: 'linux', arch: 'x86_64', docker_available: true }; },
    };
  })()`
}

/**
 * Update mock state and re-inject into page.
 */
export async function updateMockState(
  page: Page,
  updater: (state: MockState) => void,
  currentState: MockState
): Promise<void> {
  updater(currentState)
  await injectTauriMock(page, currentState)
}
