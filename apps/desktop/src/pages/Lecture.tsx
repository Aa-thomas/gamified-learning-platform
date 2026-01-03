import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

interface LectureData {
  id: string;
  title: string;
  content: string;
  difficulty: string;
  xp_reward: number;
}

interface CompletionResult {
  xp_earned: number;
  new_total_xp: number;
  new_level: number;
  unlocked_nodes: string[];
}

export function Lecture() {
  const { lectureId } = useParams<{ lectureId: string }>();
  const navigate = useNavigate();
  const [lecture, setLecture] = useState<LectureData | null>(null);
  const [loading, setLoading] = useState(true);
  const [startTime] = useState(Date.now());
  const [scrollProgress, setScrollProgress] = useState(0);
  const [timeSpent, setTimeSpent] = useState(0);
  const [completing, setCompleting] = useState(false);

  useEffect(() => {
    loadLecture();
    startLecture();

    // Track time spent
    const interval = setInterval(() => {
      if (document.visibilityState === 'visible') {
        setTimeSpent(Date.now() - startTime);
      }
    }, 1000);

    // Track scroll progress
    const handleScroll = () => {
      const windowHeight = window.innerHeight;
      const documentHeight = document.documentElement.scrollHeight;
      const scrollTop = window.scrollY;
      const progress = (scrollTop + windowHeight) / documentHeight;
      setScrollProgress(progress);
    };

    window.addEventListener('scroll', handleScroll);

    return () => {
      clearInterval(interval);
      window.removeEventListener('scroll', handleScroll);
    };
  }, []);

  const loadLecture = async () => {
    try {
      const data = await invoke<LectureData>('load_lecture', { lectureId });
      setLecture(data);
      setLoading(false);
    } catch (error) {
      console.error('Failed to load lecture:', error);
      setLoading(false);
    }
  };

  const startLecture = async () => {
    try {
      await invoke('start_lecture', { lectureId });
    } catch (error) {
      console.error('Failed to start lecture:', error);
    }
  };

  const handleComplete = async () => {
    if (!lecture) return;

    if (timeSpent < 30000) {
      alert('Please spend at least 30 seconds reading the lecture');
      return;
    }

    if (scrollProgress < 0.95) {
      const confirmed = confirm('You haven\'t scrolled to the bottom. Mark complete anyway?');
      if (!confirmed) return;
    }

    setCompleting(true);
    try {
      const result = await invoke<CompletionResult>('complete_lecture', {
        request: {
          lecture_id: lectureId,
          time_spent_ms: timeSpent,
          difficulty: lecture.difficulty,
        },
      });

      alert(`Lecture complete! You earned ${result.xp_earned} XP!`);
      navigate('/');
    } catch (error) {
      console.error('Failed to complete lecture:', error);
      alert('Failed to complete lecture');
    } finally {
      setCompleting(false);
    }
  };

  if (loading) {
    return <div className="p-8">Loading lecture...</div>;
  }

  if (!lecture) {
    return <div className="p-8">Lecture not found</div>;
  }

  const canComplete = scrollProgress >= 0.95 && timeSpent >= 30000;
  const minutesSpent = Math.floor(timeSpent / 60000);
  const secondsSpent = Math.floor((timeSpent % 60000) / 1000);

  return (
    <div className="max-w-4xl mx-auto p-8">
      <div className="mb-6">
        <button
          onClick={() => navigate('/')}
          className="text-blue-600 hover:text-blue-800"
        >
          ← Back
        </button>
      </div>

      <div className="mb-6">
        <h1 className="text-3xl font-bold mb-2">{lecture.title}</h1>
        <div className="flex gap-4 text-sm text-gray-600">
          <span>Difficulty: {lecture.difficulty}</span>
          <span>XP Reward: {lecture.xp_reward}</span>
          <span>Time: {minutesSpent}:{secondsSpent.toString().padStart(2, '0')}</span>
        </div>
        <div className="mt-2">
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className="bg-blue-600 h-2 rounded-full transition-all"
              style={{ width: `${scrollProgress * 100}%` }}
            />
          </div>
          <p className="text-xs text-gray-500 mt-1">
            Scroll progress: {Math.round(scrollProgress * 100)}%
          </p>
        </div>
      </div>

      <div className="prose max-w-none mb-8">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>
          {lecture.content}
        </ReactMarkdown>
      </div>

      <div className="sticky bottom-0 bg-white border-t p-4 flex justify-between items-center">
        <button
          onClick={() => navigate('/')}
          className="px-4 py-2 text-gray-700 hover:text-gray-900"
        >
          Cancel
        </button>
        <button
          onClick={handleComplete}
          disabled={!canComplete || completing}
          className={`px-6 py-2 rounded ${
            canComplete && !completing
              ? 'bg-green-600 hover:bg-green-700 text-white'
              : 'bg-gray-300 text-gray-500 cursor-not-allowed'
          }`}
        >
          {completing ? 'Completing...' : 'Mark Complete'}
        </button>
      </div>
    </div>
  );
}
