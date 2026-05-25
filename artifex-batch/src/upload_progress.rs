//
// Copyright (C) 2022-2026 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_rpc::{TransferChunk, UploadRequest, upload_request::Payload};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio_stream::Stream;

/// Wrapper stream that adds progress tracking to upload streams.
///
/// This wrapper adds progress reporting to any stream of `UploadRequest` items.
/// The progress callback is called as items are processed by the stream.
///
/// # Async Safety
///
/// The progress callback is invoked from within the async stream's `poll_next` method.
/// Callbacks should avoid blocking operations. Use `tokio::spawn_blocking` for
/// synchronous I/O operations like printing or file access.
pub struct UploadProgress<S> {
    inner: S,
    file_size: u64,
    bytes_uploaded: u64,
    progress_callback: Option<Arc<dyn Fn(u8) + Send + Sync>>,
}

impl<S> UploadProgress<S>
where
    S: Stream<Item = UploadRequest> + Unpin,
{
    /// Create a new upload progress tracker
    ///
    /// # Arguments
    /// * `inner` - The underlying upload request stream
    /// * `file_size` - Total size of the file being uploaded in bytes
    /// * `progress_callback` - Optional callback that receives progress percentage (0-100)
    ///
    /// # Async Context
    ///
    /// The callback is called from within the stream's `poll_next` method, which executes
    /// in an async context. If the callback performs blocking I/O operations (like printing
    /// to stdout), consider using `tokio::spawn_blocking` to avoid blocking the async runtime:
    ///
    /// ```no_run
    /// use std::sync::Arc;
    /// use tokio::task::spawn_blocking;
    ///
    /// let progress_callback = Arc::new(|progress: u8| {
    ///     spawn_blocking(move || {
    ///         println!("Progress: {}%", progress);
    ///     });
    /// });
    /// ```
    pub fn new(
        inner: S,
        file_size: u64,
        progress_callback: Option<Arc<dyn Fn(u8) + Send + Sync>>,
    ) -> Self {
        Self {
            inner,
            file_size,
            bytes_uploaded: 0,
            progress_callback,
        }
    }
}

impl<S> Stream for UploadProgress<S>
where
    S: Stream<Item = UploadRequest> + Unpin,
{
    type Item = UploadRequest;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(item)) => item,
            Poll::Ready(None) => {
                // When stream ends, report 100% progress for empty files
                if self.file_size == 0
                    && let Some(callback) = &self.progress_callback
                {
                    callback(100);
                }
                return Poll::Ready(None);
            }
            Poll::Pending => return Poll::Pending,
        };

        // Update progress for chunk items
        if let Some(Payload::Chunk(TransferChunk { data })) = &item.payload {
            self.bytes_uploaded += data.len() as u64;

            // Calculate progress percentage (0-100)
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let progress = if self.file_size > 0 {
                ((self.bytes_uploaded as f64 / self.file_size as f64) * 100.0) as u8
            } else {
                100 // Empty file is 100% uploaded
            };

            if let Some(callback) = &self.progress_callback {
                callback(progress);
            }
        }

        Poll::Ready(Some(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_rpc::{Digest, DigestKind, TransferEpilogue, TransferPrologue};
    use futures_util::StreamExt;
    use std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    };

    #[tokio::test]
    async fn test_progress_tracking() {
        let progress_values = Arc::new(AtomicU8::new(0));
        let callback = Arc::new({
            let progress_values = progress_values.clone();
            move |progress| {
                progress_values.store(progress, Ordering::SeqCst);
            }
        });

        // Create mock stream that simulates uploading a 1000-byte file
        let mock_stream = futures_util::stream::iter(vec![
            UploadRequest {
                payload: Some(Payload::Prologue(TransferPrologue {
                    file_name: "test.txt".to_string(),
                    file_size: 1000,
                })),
            },
            UploadRequest {
                payload: Some(Payload::Chunk(TransferChunk {
                    data: vec![0; 500], // 50% progress
                })),
            },
            UploadRequest {
                payload: Some(Payload::Chunk(TransferChunk {
                    data: vec![0; 300], // 80% progress
                })),
            },
            UploadRequest {
                payload: Some(Payload::Epilogue(TransferEpilogue {
                    digest: Some(Digest {
                        kind: DigestKind::Blake3.into(),
                        data: vec![0; 32],
                    }),
                })),
            },
        ]);

        let progress_stream = UploadProgress::new(mock_stream, 1000, Some(callback));

        // Consume the stream
        let items: Vec<_> = progress_stream.collect().await;

        // Verify final progress was 80%
        assert_eq!(progress_values.load(Ordering::SeqCst), 80);
        assert_eq!(items.len(), 4);
    }

    #[tokio::test]
    async fn test_empty_file() {
        let progress_values = Arc::new(AtomicU8::new(0));
        let callback = Arc::new({
            let progress_values = progress_values.clone();
            move |progress| {
                progress_values.store(progress, Ordering::SeqCst);
            }
        });

        // Empty file stream
        let mock_stream = futures_util::stream::iter(vec![
            UploadRequest {
                payload: Some(Payload::Prologue(TransferPrologue {
                    file_name: "empty.txt".to_string(),
                    file_size: 0,
                })),
            },
            UploadRequest {
                payload: Some(Payload::Epilogue(TransferEpilogue {
                    digest: Some(Digest {
                        kind: DigestKind::Blake3.into(),
                        data: vec![0; 32],
                    }),
                })),
            },
        ]);

        let progress_stream = UploadProgress::new(mock_stream, 0, Some(callback));

        let items: Vec<_> = progress_stream.collect().await;

        // Empty file should report 100% progress
        assert_eq!(progress_values.load(Ordering::SeqCst), 100);
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn test_no_callback() {
        // Test that it works without a callback
        let mock_stream = futures_util::stream::iter(vec![
            UploadRequest {
                payload: Some(Payload::Prologue(TransferPrologue {
                    file_name: "test.txt".to_string(),
                    file_size: 100,
                })),
            },
            UploadRequest {
                payload: Some(Payload::Chunk(TransferChunk { data: vec![0; 50] })),
            },
        ]);

        let progress_stream = UploadProgress::new(
            mock_stream,
            100,
            None, // No callback
        );

        // Should work fine without callback
        let items: Vec<_> = progress_stream.collect().await;
        assert_eq!(items.len(), 2);
    }
}
