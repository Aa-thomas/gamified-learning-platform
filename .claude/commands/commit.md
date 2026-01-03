---
description: Automatically commit and push changes to GitHub with conventional commit messages
---

You are a Git commit automation agent. Your job is to analyze changes, create conventional commit messages, and push to GitHub.

# Instructions

1. **Check Current Status**
   - Run `git status` to see all changes
   - Run `git diff` to see detailed changes (if manageable size)
   - Run `git log -1 --format='%s'` to see the last commit message style

2. **Analyze Changes**
   - Review all modified, added, and deleted files
   - Understand the nature of the changes
   - Identify the primary purpose of the changes

3. **Create Conventional Commit Message**

   Use the Conventional Commits specification:

   **Format**: `type(scope): description`

   **Types**:
   - `feat`: New feature
   - `fix`: Bug fix
   - `docs`: Documentation only changes
   - `style`: Changes that don't affect code meaning (formatting, etc.)
   - `refactor`: Code change that neither fixes a bug nor adds a feature
   - `perf`: Performance improvement
   - `test`: Adding or updating tests
   - `build`: Changes to build system or dependencies
   - `ci`: Changes to CI configuration
   - `chore`: Other changes that don't modify src or test files
   - `revert`: Reverts a previous commit

   **Scope** (optional): The area of codebase affected (e.g., parser, api, ui)

   **Description**: Short summary in present tense, lowercase, no period

   **Examples**:
   - `feat(docker): add Docker runner prototype with edge case testing`
   - `docs(phase-0): complete risk validation milestone documentation`
   - `refactor(gamification): optimize XP calculation formulas`
   - `fix(llm): resolve grading consistency issues`
   - `chore: update dependencies to latest versions`

4. **Create Multi-line Commit If Needed**

   For complex changes, use multi-line format:
   ```
   type(scope): short description

   - Detailed point 1
   - Detailed point 2
   - Detailed point 3

   Co-Authored-By: Claude <noreply@anthropic.com>
   ```

5. **Execute Git Operations**
   - Stage all changes: `git add .` (or specific files if appropriate)
   - Commit with the generated message using heredoc format
   - Push to remote: `git push`

   Use heredoc for commit messages to ensure proper formatting:
   ```bash
   git commit -m "$(cat <<'EOF'
   feat(scope): description

   Additional details if needed

   Co-Authored-By: Claude <noreply@anthropic.com>
   EOF
   )"
   ```

6. **Report Results**
   - Show the commit message that was created
   - Confirm successful commit and push
   - Display any errors if they occur

# Important Rules

- NEVER commit sensitive files (.env, credentials, secrets)
- Check for large files (>10MB) and warn before committing
- If there are no changes to commit, inform the user
- If remote is ahead of local, pull first or inform user
- Always add `Co-Authored-By: Claude <noreply@anthropic.com>` to commit body
- Keep the first line under 72 characters
- Use present tense ("add feature" not "added feature")
- Use imperative mood ("move cursor to..." not "moves cursor to...")

# Error Handling

If you encounter:
- **Merge conflicts**: Inform user and don't proceed
- **Untracked files to ignore**: Ask user if they should be added to .gitignore
- **Large files**: Warn and ask for confirmation
- **No remote configured**: Inform user to set up remote first
- **Authentication issues**: Provide troubleshooting guidance

# Example Flow

1. Run git status and diff
2. Analyze: "User completed Phase 0 prototypes with Docker runner, LLM grading, and gamification"
3. Create message: `feat(prototypes): complete Phase 0 risk validation milestones`
4. Add body with details about the three prototypes
5. Stage, commit, and push
6. Report success

Now proceed with analyzing the current changes and committing them.
