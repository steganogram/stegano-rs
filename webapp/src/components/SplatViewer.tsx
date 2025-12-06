import React, { useEffect, useState } from 'react';
import JSZip from 'jszip';

interface SplatViewerProps {
    fileData: Uint8Array;
    fileName: string;
    onClose: () => void;
}

interface ExtractedFile {
    name: string;
    url: string;
    type: 'image' | 'video' | 'html' | 'other';
}

const SplatViewer: React.FC<SplatViewerProps> = ({ fileData, fileName, onClose }) => {
    const [loading, setLoading] = useState(true);
    const [viewerUrl, setViewerUrl] = useState<string | null>(null);
    const [htmlContent, setHtmlContent] = useState<string | null>(null); // Store string for document.write
    const [mediaFiles, setMediaFiles] = useState<ExtractedFile[]>([]);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const processZip = async () => {
            try {
                setLoading(true);
                const zip = new JSZip();
                const loadedZip = await zip.loadAsync(fileData);

                let splatHtmlFile: JSZip.JSZipObject | null = null;
                const images: ExtractedFile[] = [];

                const filePromises: Promise<void>[] = [];

                loadedZip.forEach((relativePath, zipEntry) => {
                    filePromises.push((async () => {
                        if (zipEntry.dir) return;

                        const lowerName = zipEntry.name.toLowerCase();

                        if (lowerName.endsWith('.html')) {
                            const text = await zipEntry.async('string');
                            if (text.includes('<title>SuperSplat') || text.includes('SuperSplat Viewer') || text.includes('<!DOCTYPE html>')) {
                                splatHtmlFile = zipEntry;
                            }
                        } else if (lowerName.match(/\.(jpg|jpeg|png|gif|webp)$/)) {
                            const blob = await zipEntry.async('blob');
                            const url = URL.createObjectURL(blob);
                            images.push({ name: zipEntry.name, url, type: 'image' });
                        } else if (lowerName.match(/\.(mp4|webm)$/)) {
                            const blob = await zipEntry.async('blob');
                            const url = URL.createObjectURL(blob);
                            images.push({ name: zipEntry.name, url, type: 'video' });
                        }
                    })());
                });

                await Promise.all(filePromises);

                if (splatHtmlFile) {
                    const entry = splatHtmlFile as JSZip.JSZipObject;
                    const text = await entry.async('string');
                    setHtmlContent(text);

                    // Create Blob with explicit charset for iframe backup
                    const blob = new Blob([text], { type: 'text/html; charset=utf-8' });
                    const url = URL.createObjectURL(blob);
                    setViewerUrl(url);
                }

                setMediaFiles(images);
                setLoading(false);

            } catch (err) {
                console.error("Failed to unzip:", err);
                setError("Failed to extract Zip file.");
                setLoading(false);
            }
        };

        const processHtml = () => {
            try {
                setLoading(true);
                const decoder = new TextDecoder('utf-8');
                const text = decoder.decode(fileData);
                setHtmlContent(text);

                const blob = new Blob([fileData], { type: 'text/html; charset=utf-8' });
                const url = URL.createObjectURL(blob);
                setViewerUrl(url);
                setLoading(false);
            } catch (err) {
                setError("Failed to load HTML file.");
                setLoading(false);
            }
        };

        if (fileName.toLowerCase().endsWith('.zip')) {
            processZip();
        } else if (fileName.toLowerCase().endsWith('.html')) {
            processHtml();
        } else {
            setError("Not a recognized archive format for Viewer.");
            setLoading(false);
        }

        return () => {
            if (viewerUrl) URL.revokeObjectURL(viewerUrl);
            mediaFiles.forEach(f => URL.revokeObjectURL(f.url));
        };
    }, [fileData, fileName]);

    // Robust "New Tab" specifically using document.write
    // This avoids Blob URL Origin issues by writing directly to the same-origin window
    const openInNewTab = () => {
        if (htmlContent) {
            const win = window.open('', '_blank');
            if (win) {
                win.document.open();
                win.document.write(htmlContent);
                win.document.close();
            } else {
                alert("Pop-up blocked! Please allow pop-ups for this site.");
            }
        } else if (viewerUrl) {
            window.open(viewerUrl, '_blank');
        }
    };

    const downloadExtracted = () => {
        if (htmlContent) {
            const blob = new Blob([htmlContent], { type: 'text/html; charset=utf-8' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = "extracted_viewer.html";
            a.click();
            URL.revokeObjectURL(url);
        }
        mediaFiles.forEach(f => {
            const a = document.createElement('a');
            a.href = f.url;
            a.download = f.name;
            a.click();
        });
    };

    if (loading) return <div className="loading-spinner">Loading Content...</div>;
    if (error) return <div className="error-msg">{error}</div>;

    return (
        <div className="splat-viewer-overlay">
            <div className="splat-viewer-content">
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                    <h3 style={{ margin: 0 }}>Content Viewer</h3>
                    <div>
                        {(htmlContent || viewerUrl) && (
                            <button className="btn" onClick={openInNewTab} style={{ marginRight: '0.5rem', width: 'auto', padding: '0.5rem 1rem', background: '#03dac6', color: '#000' }}>
                                ↗ Open Fullscreen
                            </button>
                        )}
                        <button className="btn" onClick={downloadExtracted} style={{ marginRight: '1rem', width: 'auto', padding: '0.5rem 1rem' }}>
                            Download
                        </button>
                        <button className="close-btn" onClick={onClose} style={{ position: 'static' }}>
                            Close
                        </button>
                    </div>
                </div>

                {viewerUrl && (
                    <div className="iframe-container" style={{ position: 'relative' }}>
                        <iframe
                            src={viewerUrl}
                            title="Splat Viewer"
                            className="splat-iframe"
                        />
                    </div>
                )}

                {mediaFiles.length > 0 && (
                    <div className="media-gallery">
                        <h3>Extracted Media</h3>
                        <div className="gallery-grid">
                            {mediaFiles.map((file, i) => (
                                <div key={i} className="gallery-item">
                                    {file.type === 'image' ? (
                                        <img src={file.url} alt={file.name} loading="lazy" />
                                    ) : (
                                        <video src={file.url} controls />
                                    )}
                                    <p>{file.name}</p>
                                </div>
                            ))}
                        </div>
                    </div>
                )}

                {!viewerUrl && mediaFiles.length === 0 && (
                    <div style={{ textAlign: 'center', padding: '2rem' }}>
                        <p>No previewable content found.</p>
                    </div>
                )}
            </div>
        </div>
    );
};

export default SplatViewer;
