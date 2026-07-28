// Hash utilities (SHA-256 and MD5)
import crypto from 'crypto'

export function computeSha256(data: Buffer): string {
  return crypto.createHash('sha256').update(data).digest('hex')
}

export function computeMd5(data: Buffer): string {
  return crypto.createHash('md5').update(data).digest('hex')
}

export async function verifyFileHash(
  filePath: string,
  expectedSha256: string,
): Promise<'intact' | 'altered' | 'missing'> {
  try {
    const fs = await import('fs')
    const content = fs.readFileSync(filePath)
    const actual = computeSha256(content)
    return actual === expectedSha256 ? 'intact' : 'altered'
  } catch {
    return 'missing'
  }
}
