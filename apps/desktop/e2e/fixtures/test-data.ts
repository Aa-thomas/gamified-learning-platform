import { MockContentTree, MockUser, MockNodeProgress, MockBadge, MockState, createMockState } from '../mocks/tauri'

/**
 * Test data factories for E2E tests.
 * These provide realistic data for testing various scenarios.
 */

/**
 * Create a mock user with customizable properties.
 */
export function createUser(overrides: Partial<MockUser> = {}): MockUser {
  const now = new Date().toISOString()
  return {
    id: Date.now(),
    username: 'TestUser',
    xp: 0,
    level: 1,
    streak_days: 0,
    last_active_date: now,
    created_at: now,
    ...overrides,
  }
}

/**
 * Create an experienced user (mid-level with XP and streak).
 */
export function createExperiencedUser(): MockUser {
  return createUser({
    username: 'ExperiencedUser',
    xp: 500,
    level: 2,
    streak_days: 7,
  })
}

/**
 * Create a user about to level up.
 */
export function createUserNearLevelUp(): MockUser {
  return createUser({
    username: 'AlmostThere',
    xp: 270,  // 13 XP away from level 2 (283)
    level: 1,
    streak_days: 3,
  })
}

/**
 * Create a sample content tree for testing.
 */
export function createContentTree(): MockContentTree {
  return {
    weeks: [
      {
        id: 'week1',
        title: 'Week 1: Getting Started',
        days: [
          {
            id: 'week1-day1',
            title: 'Day 1: Introduction',
            nodes: [
              {
                id: 'week1-day1-lecture',
                title: 'Introduction to Rust',
                node_type: 'lecture',
                description: 'Learn the basics of Rust programming language',
                difficulty: 'Easy',
                xp_reward: 25,
              },
              {
                id: 'week1-day1-quiz',
                title: 'Rust Basics Quiz',
                node_type: 'quiz',
                description: 'Test your understanding of Rust basics',
                difficulty: 'Easy',
                xp_reward: 50,
              },
            ],
          },
          {
            id: 'week1-day2',
            title: 'Day 2: Ownership',
            nodes: [
              {
                id: 'week1-day2-lecture',
                title: 'Understanding Ownership',
                node_type: 'lecture',
                description: "Rust's unique ownership system",
                difficulty: 'Medium',
                xp_reward: 38,
              },
              {
                id: 'week1-day2-quiz',
                title: 'Ownership Quiz',
                node_type: 'quiz',
                description: 'Test your understanding of ownership',
                difficulty: 'Medium',
                xp_reward: 75,
              },
              {
                id: 'week1-day2-challenge',
                title: 'Ownership Challenge',
                node_type: 'challenge',
                description: 'Practice ownership concepts',
                difficulty: 'Medium',
                xp_reward: 150,
              },
            ],
          },
        ],
      },
      {
        id: 'week2',
        title: 'Week 2: Advanced Concepts',
        days: [
          {
            id: 'week2-day1',
            title: 'Day 1: Lifetimes',
            nodes: [
              {
                id: 'week2-day1-lecture',
                title: 'Understanding Lifetimes',
                node_type: 'lecture',
                description: 'Learn about Rust lifetimes',
                difficulty: 'Hard',
                xp_reward: 50,
              },
            ],
          },
        ],
      },
    ],
  }
}

/**
 * Create node progress entries.
 */
export function createProgress(nodeId: string, status: MockNodeProgress['status'], xpEarned = 0): MockNodeProgress {
  const now = new Date().toISOString()
  return {
    id: Math.floor(Math.random() * 10000),
    user_id: 1,
    node_id: nodeId,
    status,
    started_at: status !== 'locked' ? now : undefined,
    completed_at: status === 'completed' ? now : undefined,
    xp_earned: xpEarned,
    attempts: status === 'completed' ? 1 : 0,
  }
}

/**
 * Create progress for a user who completed the first lecture.
 */
export function createProgressAfterFirstLecture(): Record<string, MockNodeProgress> {
  return {
    'week1-day1-lecture': createProgress('week1-day1-lecture', 'completed', 25),
  }
}

/**
 * Create badges for testing.
 */
export function createBadges(): MockBadge[] {
  return [
    {
      id: 'first-steps',
      title: 'First Steps',
      description: 'Complete your first lecture',
      icon: '🦀',
      unlocked: false,
    },
    {
      id: 'quiz-master',
      title: 'Quiz Master',
      description: 'Score 100% on a quiz',
      icon: '🏆',
      unlocked: false,
    },
    {
      id: 'streak-starter',
      title: 'Streak Starter',
      description: 'Maintain a 7-day streak',
      icon: '🔥',
      unlocked: false,
    },
    {
      id: 'level-up',
      title: 'Level Up!',
      description: 'Reach level 2',
      icon: '⬆️',
      unlocked: false,
    },
  ]
}

/**
 * Pre-configured mock states for common test scenarios.
 */
export const scenarios = {
  /**
   * Fresh state - no user, no progress.
   */
  fresh: (): MockState => createMockState({
    contentTree: createContentTree(),
    badges: createBadges(),
  }),

  /**
   * New user just created.
   */
  newUser: (): MockState => createMockState({
    user: createUser(),
    contentTree: createContentTree(),
    badges: createBadges(),
  }),

  /**
   * User with some progress (completed first lecture).
   */
  withProgress: (): MockState => createMockState({
    user: createUser({ xp: 25 }),
    progress: createProgressAfterFirstLecture(),
    contentTree: createContentTree(),
    badges: createBadges().map((b) =>
      b.id === 'first-steps' ? { ...b, unlocked: true, unlocked_at: new Date().toISOString() } : b
    ),
  }),

  /**
   * Experienced user with significant progress.
   */
  experienced: (): MockState => createMockState({
    user: createExperiencedUser(),
    progress: {
      'week1-day1-lecture': createProgress('week1-day1-lecture', 'completed', 25),
      'week1-day1-quiz': createProgress('week1-day1-quiz', 'completed', 65),
      'week1-day2-lecture': createProgress('week1-day2-lecture', 'completed', 38),
      'week1-day2-quiz': createProgress('week1-day2-quiz', 'completed', 82),
      'week1-day2-challenge': createProgress('week1-day2-challenge', 'completed', 165),
    },
    masteryScores: {
      'rust-basics': 0.65,
      'ownership': 0.45,
    },
    contentTree: createContentTree(),
    badges: createBadges().map((b) =>
      ['first-steps', 'streak-starter', 'level-up'].includes(b.id)
        ? { ...b, unlocked: true, unlocked_at: new Date().toISOString() }
        : b
    ),
  }),

  /**
   * User about to level up (for testing level-up flow).
   */
  nearLevelUp: (): MockState => createMockState({
    user: createUserNearLevelUp(),
    progress: {
      'week1-day1-lecture': createProgress('week1-day1-lecture', 'completed', 25),
      'week1-day1-quiz': createProgress('week1-day1-quiz', 'completed', 65),
      'week1-day2-lecture': createProgress('week1-day2-lecture', 'completed', 38),
    },
    contentTree: createContentTree(),
    badges: createBadges().map((b) =>
      b.id === 'first-steps' ? { ...b, unlocked: true, unlocked_at: new Date().toISOString() } : b
    ),
  }),
}

/**
 * Expected values for golden tests.
 * These are the "correct" values that tests should verify against.
 */
export const expectedValues = {
  xpForLevel2: 283,
  xpForLevel3: 520,
  xpForLevel5: 1118,
  xpForLevel10: 3162,

  // Quiz XP calculations (Easy quiz, 66.7% score, no streak)
  quizXpEasy67Percent: 55, // 50 * 1.0 * 1.0 * 1.1 ≈ 55

  // Lecture XP (Easy, no streak)
  lectureXpEasyNoStreak: 25,

  // Streak multipliers
  streakMultiplier7Days: 1.1,
  streakMultiplier14Days: 1.2,
  streakMultiplier31Days: 1.5,
}
