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
    const [srcDoc, setSrcDoc] = useState<string | null>(null);
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

                // 1. Scan files
                const filePromises: Promise<void>[] = [];

                loadedZip.forEach((relativePath, zipEntry) => {
                    filePromises.push((async () => {
                        if (zipEntry.dir) return;

                        const lowerName = zipEntry.name.toLowerCase();

                        // Check for SPLAT html inside zip
                        if (lowerName.endsWith('.html')) {
                            const text = await zipEntry.async('string');
                            // Relaxed detection: if it's an HTML file in a zip, it might be the viewer
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
                    setSrcDoc(text);
                    setViewerUrl(null);
                }

                setMediaFiles(images);
                setLoading(false);

            } catch (err) {
                console.error("Failed to unzip:", err);
                setError("Failed to extract Zip file. It might be corrupt or not a Zip.");
                setLoading(false);
            }
        };

        const processHtml = () => {
            try {
                setLoading(true);
                // Convert Uint8Array to string manually to avoid type issues with Blob
                const decoder = new TextDecoder('utf-8');
                const text = decoder.decode(fileData);
                setSrcDoc(text);
                setViewerUrl(null);
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
            // Cleanup URLs
            if (viewerUrl) URL.revokeObjectURL(viewerUrl);
            mediaFiles.forEach(f => URL.revokeObjectURL(f.url));
        };
    }, [fileData, fileName]);

    if (loading) return <div className="loading-spinner">Loading Content...</div>;
    if (error) return <div className="error-msg">{error}</div>;

    return (
        <div className="splat-viewer-overlay">
            <div className="splat-viewer-content">
                <button className="close-btn" onClick={onClose}>&times; Close Viewer</button>

                {(viewerUrl || srcDoc) && (
                    <div className="iframe-container">
                        <h3>Gaussian Splat Viewer</h3>
                        <iframe
                            src={viewerUrl || undefined}
                            srcDoc={srcDoc || undefined}
                            title="Splat Viewer"
                            // Relaxed sandbox for WebGL and scripts
                            sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"
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

                {!viewerUrl && !srcDoc && mediaFiles.length === 0 && (
                    <div style={{ textAlign: 'center', padding: '2rem' }}>
                        <p>No previewable content found in archive.</p>
                        <p style={{ fontSize: '0.8rem', color: '#888' }}>
                            Ensure the Zip contains images or an HTML viewer (e.g. SuperSplat).
                        </p>
                    </div>
                )}
            </div>
        </div>
    );
};

export default SplatViewer;
