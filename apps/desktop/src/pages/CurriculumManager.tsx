import { useEffect, useState } from 'react'
import { useCurriculumStore, ValidationResponse } from '@/stores/curriculumStore'
import { open } from '@tauri-apps/plugin-dialog'

export function CurriculumManager() {
  const {
    curricula,
    activeCurriculum,
    loading,
    error,
    fetchCurricula,
    fetchActiveCurriculum,
    validateCurriculum,
    importCurriculum,
    switchCurriculum,
    deleteCurriculum,
  } = useCurriculumStore()

  const [validation, setValidation] = useState<ValidationResponse | null>(null)
  const [selectedPath, setSelectedPath] = useState<string | null>(null)
  const [importError, setImportError] = useState<string | null>(null)
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null)

  useEffect(() => {
    fetchCurricula()
    fetchActiveCurriculum()
  }, [fetchCurricula, fetchActiveCurriculum])

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Content Pack Folder',
      })

      if (selected && typeof selected === 'string') {
        setSelectedPath(selected)
        setImportError(null)
        setValidation(null)

        const result = await validateCurriculum(selected)
        setValidation(result)
      }
    } catch (err) {
      setImportError(String(err))
    }
  }

  const handleImport = async (setActive: boolean) => {
    if (!selectedPath) return

    try {
      const result = await importCurriculum(selectedPath, setActive)
      if (result.success) {
        setSelectedPath(null)
        setValidation(null)
        setImportError(null)
      } else {
        setImportError(result.error ?? 'Import failed')
      }
    } catch (err) {
      setImportError(String(err))
    }
  }

  const handleSwitch = async (curriculumId: string) => {
    try {
      await switchCurriculum(curriculumId)
    } catch (err) {
      setImportError(String(err))
    }
  }

  const handleDelete = async (curriculumId: string, deleteProgress: boolean) => {
    try {
      await deleteCurriculum(curriculumId, deleteProgress)
      setDeleteConfirm(null)
    } catch (err) {
      setImportError(String(err))
    }
  }

  const formatMinutes = (minutes: number) => {
    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60
    if (hours > 0) {
      return `${hours}h ${mins}m`
    }
    return `${mins}m`
  }

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">Curriculum Manager</h1>

      {error && (
        <div className="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
          {error}
        </div>
      )}

      {/* Import Section */}
      <div className="bg-white rounded-lg shadow mb-6">
        <div className="p-6 border-b">
          <h2 className="text-lg font-semibold">Import New Curriculum</h2>
          <p className="text-sm text-gray-600 mt-1">
            Select a folder containing a curriculum content pack with a manifest.json file.
          </p>
        </div>

        <div className="p-6">
          <button
            onClick={handleSelectFolder}
            disabled={loading}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
          >
            {loading ? 'Loading...' : 'Select Folder'}
          </button>

          {selectedPath && (
            <div className="mt-4 p-4 bg-gray-50 rounded-lg">
              <p className="text-sm font-medium text-gray-700">Selected:</p>
              <p className="text-sm text-gray-600 font-mono truncate">{selectedPath}</p>
            </div>
          )}

          {validation && (
            <div className="mt-4">
              {validation.is_valid ? (
                <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
                  <div className="flex items-center gap-2 text-green-700 font-medium">
                    <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    Valid Content Pack
                  </div>

                  <div className="mt-3 grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="font-medium">Name:</span> {validation.name}
                    </div>
                    <div>
                      <span className="font-medium">Version:</span> {validation.version}
                    </div>
                    {validation.author && (
                      <div>
                        <span className="font-medium">Author:</span> {validation.author}
                      </div>
                    )}
                  </div>

                  {validation.stats && (
                    <div className="mt-3 pt-3 border-t border-green-200">
                      <p className="text-sm font-medium text-green-700 mb-2">Content Stats:</p>
                      <div className="grid grid-cols-3 gap-2 text-sm text-green-600">
                        <div>{validation.stats.total_weeks} weeks</div>
                        <div>{validation.stats.total_days} days</div>
                        <div>{validation.stats.total_nodes} nodes</div>
                        <div>{validation.stats.lectures} lectures</div>
                        <div>{validation.stats.quizzes} quizzes</div>
                        <div>{validation.stats.challenges} challenges</div>
                        <div>{validation.stats.total_xp.toLocaleString()} XP</div>
                        <div>{formatMinutes(validation.stats.total_estimated_minutes)}</div>
                      </div>
                    </div>
                  )}

                  {validation.warnings.length > 0 && (
                    <div className="mt-3 pt-3 border-t border-yellow-200">
                      <p className="text-sm font-medium text-yellow-700">Warnings:</p>
                      <ul className="text-sm text-yellow-600 list-disc list-inside">
                        {validation.warnings.map((w, i) => (
                          <li key={i}>{w}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  <div className="mt-4 flex gap-3">
                    <button
                      onClick={() => handleImport(true)}
                      disabled={loading}
                      className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50 transition-colors"
                    >
                      Import & Set Active
                    </button>
                    <button
                      onClick={() => handleImport(false)}
                      disabled={loading}
                      className="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 disabled:opacity-50 transition-colors"
                    >
                      Import Only
                    </button>
                  </div>
                </div>
              ) : (
                <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
                  <div className="flex items-center gap-2 text-red-700 font-medium">
                    <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                    </svg>
                    Invalid Content Pack
                  </div>
                  <ul className="mt-2 text-sm text-red-600 list-disc list-inside">
                    {validation.errors.map((e, i) => (
                      <li key={i}>{e}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {importError && (
            <div className="mt-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
              {importError}
            </div>
          )}
        </div>
      </div>

      {/* Active Curriculum */}
      {activeCurriculum && (
        <div className="bg-white rounded-lg shadow mb-6">
          <div className="p-6 border-b">
            <h2 className="text-lg font-semibold">Active Curriculum</h2>
          </div>
          <div className="p-6">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-medium text-lg">{activeCurriculum.name}</h3>
                <p className="text-sm text-gray-500">
                  Version {activeCurriculum.version}
                  {activeCurriculum.author && ` • by ${activeCurriculum.author}`}
                </p>
              </div>
              <span className="px-3 py-1 bg-green-100 text-green-700 text-sm font-medium rounded-full">
                Active
              </span>
            </div>
            {activeCurriculum.description && (
              <p className="mt-2 text-sm text-gray-600">{activeCurriculum.description}</p>
            )}
          </div>
        </div>
      )}

      {/* Curriculum List */}
      <div className="bg-white rounded-lg shadow">
        <div className="p-6 border-b">
          <h2 className="text-lg font-semibold">Imported Curricula</h2>
        </div>

        {curricula.length === 0 ? (
          <div className="p-6 text-center text-gray-500">
            No curricula imported yet. Import a content pack to get started.
          </div>
        ) : (
          <div className="divide-y">
            {curricula.map((curriculum) => (
              <div key={curriculum.id} className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <h3 className="font-medium">{curriculum.name}</h3>
                    <p className="text-sm text-gray-500">
                      Version {curriculum.version}
                      {curriculum.author && ` • by ${curriculum.author}`}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    {curriculum.is_active ? (
                      <span className="px-3 py-1 bg-green-100 text-green-700 text-sm font-medium rounded-full">
                        Active
                      </span>
                    ) : (
                      <button
                        onClick={() => handleSwitch(curriculum.id)}
                        disabled={loading}
                        className="px-3 py-1 text-sm text-blue-600 border border-blue-300 rounded-lg hover:bg-blue-50 transition-colors disabled:opacity-50"
                      >
                        Set Active
                      </button>
                    )}
                    {deleteConfirm === curriculum.id ? (
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => handleDelete(curriculum.id, false)}
                          className="px-3 py-1 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50"
                        >
                          Delete Only
                        </button>
                        <button
                          onClick={() => handleDelete(curriculum.id, true)}
                          className="px-3 py-1 text-sm text-white bg-red-600 rounded-lg hover:bg-red-700"
                        >
                          + Progress
                        </button>
                        <button
                          onClick={() => setDeleteConfirm(null)}
                          className="px-3 py-1 text-sm text-gray-600 border border-gray-300 rounded-lg hover:bg-gray-50"
                        >
                          Cancel
                        </button>
                      </div>
                    ) : (
                      <button
                        onClick={() => setDeleteConfirm(curriculum.id)}
                        disabled={loading}
                        className="px-3 py-1 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50 transition-colors disabled:opacity-50"
                      >
                        Delete
                      </button>
                    )}
                  </div>
                </div>
                {curriculum.description && (
                  <p className="mt-2 text-sm text-gray-600">{curriculum.description}</p>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
