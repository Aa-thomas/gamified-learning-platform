import { cn } from '@/utils/cn'

interface SkeletonProps {
  className?: string
  width?: number | string
  height?: number | string
}

export function Skeleton({ className, width, height }: SkeletonProps) {
  return (
    <div
      data-testid="skeleton"
      className={cn(
        'animate-pulse rounded bg-gray-200 dark:bg-gray-700',
        className
      )}
      style={{
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height,
      }}
    />
  )
}

export function TextSkeleton({ lines = 3 }: { lines?: number }) {
  return (
    <div className="space-y-2">
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          className="h-4"
          width={i === lines - 1 ? '60%' : '100%'}
        />
      ))}
    </div>
  )
}

export function CardSkeleton() {
  return (
    <div
      data-testid="card-skeleton"
      className="rounded-lg border border-gray-200 dark:border-gray-700 p-4 space-y-3"
    >
      <Skeleton className="h-6 w-3/4" />
      <Skeleton className="h-4 w-full" />
      <Skeleton className="h-4 w-5/6" />
    </div>
  )
}

export function ProgressCardSkeleton() {
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 space-y-4">
      <Skeleton className="h-6 w-1/3" />
      <Skeleton className="h-4 w-full rounded-full" />
      <div className="flex justify-between">
        <Skeleton className="h-4 w-24" />
        <Skeleton className="h-4 w-16" />
      </div>
    </div>
  )
}
