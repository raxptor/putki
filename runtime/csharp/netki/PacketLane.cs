using System;

namespace Netki
{
	public interface BufferFactory
	{
		Bitstream.Buffer GetBuffer(uint minSize);
		void ReturnBuffer(Bitstream.Buffer buf);
	}

	public static class PacketLane
	{
		public struct PendingOut
		{
			public Bitstream.Buffer Source;
			public uint SeqId;
			public bool IsFinalPiece;
			public bool Reliable;
			public uint Begin, End;
			public DateTime SendTime;
			public DateTime InitialSendTime;
			public uint SendCount;
		}

		public struct PendingIn
		{
			public Bitstream.Buffer Packet;
			public uint Seq;
			public bool Reliable;
			public DateTime ArrivalTime;
		}

		public struct InProgress
		{
			public uint SeqId;
			public Bitstream.Buffer Data;
			public DateTime ArrivalTime;
			public bool IsFinalPiece;
		}

		public struct Done
		{
			public uint SeqId;
			public Lane Lane;
			public bool Reliable;
			public Bitstream.Buffer Data;
			public DateTime ArrivalTime;
			public DateTime CompletionTime;
		}

		public struct Send
		{
			public Lane Lane;
			public Bitstream.Buffer Data;
		}

		public struct ToSend
		{
			public Lane Lane;
			public Bitstream.Buffer Data;
			public bool Reliable;
			public bool CanKeepData;
		}

		public struct Statistics
		{
			public uint SendCount;
			public uint SendResends;
			public uint SendAckOnly;
			public uint SendBytes;
			public uint RecvCount;
			public uint RecvDupes;
			public uint RecvNonFinal;
			public uint RecvAckOnly;
			public uint RecvBytes;
		}

		public class Lane
		{
			// id is user data.
			public Lane(ulong id, int slots)
			{
				Out = new PendingOut[slots];
				Progress = new InProgress[slots];
				Done = new Done[slots];
				Acks = new uint[slots];
				Id = id;
				LagMsMin = 0;
				ResendMs = 200;
			}

			public ulong Id;

			public PendingOut[] Out;
			public ushort OutCount;

			public InProgress[] Progress;
			public uint ProgressHead;
			public uint ProgressTail;

			public Done[] Done;
			public uint DoneHead;
			public uint DoneTail;

			public uint[] Acks;
			public ushort AckCount;

			public uint ReliableSeq;

			public uint ResendMs;
			public uint OutgoingSeqUnreliable;
			public uint OutgoingSeqReliable;
			public uint LagMsMin;

			public Statistics Stats;
		}

		public struct IncomingInternal
		{
			public uint Seq;
			public bool IsReliable;
			public bool IsFinalPiece;
		}

		public struct Incoming
		{
			public Lane Lane;
			public Bitstream.Buffer Packet;
			public DateTime ArrivalTime;
			public IncomingInternal Internal;
			public bool CanKeepData;
		}

		public struct LaneSetup
		{
			public BufferFactory Factory;
			public uint MaxPacketSize;
			public uint ReservedHeaderBytes;
			public uint MinResendTimeMs;
		}

		// This will start reading from the buffers and modify the incoming array.
		public static void HandleIncomingPackets(LaneSetup setup, Incoming[] packets, uint packetsCount)
		{
			// Zero pass make own buffers.
			for (uint i=0;i<packetsCount;i++)
			{
				if (!packets[i].CanKeepData)
				{
					Bitstream.Buffer tmp = packets[i].Packet;
					Bitstream.Buffer copy = setup.Factory.GetBuffer((uint)tmp.buf.Length);
					Bitstream.Insert(copy, tmp);
					copy.Flip();
					packets[i].Packet = copy;
				}
			}

			// First pass extract.
			for (uint i=0;i<packetsCount;i++)
			{
				Bitstream.Buffer tmp = packets[i].Packet;
				Lane lane = packets[i].Lane;
				lane.Stats.RecvCount++;
				lane.Stats.RecvBytes += tmp.BitsLeft() / 8;

				bool reliable = Bitstream.ReadBits(tmp, 1) == 1;
				uint seq = Bitstream.ReadCompressedUint(tmp);
				packets[i].Internal.Seq = seq;
				packets[i].Internal.IsReliable = reliable;

				if (seq == 0)
				{
					lane.Stats.RecvAckOnly++;
				}

				if (reliable)
				{
					packets[i].Internal.IsFinalPiece = Bitstream.ReadBits(tmp, 1) == 1;

					if (!packets[i].Internal.IsFinalPiece)
					{
						lane.Stats.RecvNonFinal++;
					}

					if (seq != 0)
					{
						if (lane.AckCount < lane.Acks.Length)
						{						
							lane.Acks[lane.AckCount++] = seq;
						}
					}

					while (true)
					{
						uint seqAck = Bitstream.ReadCompressedUint(tmp);
						if (seqAck != 0)
						{
							for (int j=0;j!=lane.OutCount;j++)
							{								
								if (lane.Out[j].SeqId == seqAck)
								{
									double msLag = (packets[i].ArrivalTime - lane.Out[j].InitialSendTime).TotalMilliseconds;
									if (msLag > 0)
									{
										uint ms = (uint) msLag;
										if (ms < lane.LagMsMin || lane.LagMsMin == 0)
										{
											if (ms >= setup.MinResendTimeMs)
											{
												lane.LagMsMin = ms;
												lane.ResendMs = 2 * ms;
											}
										}
									}

									if (lane.Out[j].Source != null)
									{
										if (--lane.Out[j].Source.userdata == 0)
										{
											setup.Factory.ReturnBuffer(lane.Out[j].Source);
										}
										lane.Out[j].Source = null;
									}
								}
							}
							continue;
						}
						break;
					}
				}
				else
				{
					packets[i].Internal.IsFinalPiece = true;
				}

				Bitstream.SyncByte(tmp);
			}

			// Unreliable packets, these are placed in the out queue after reading out the sequence and mode.
			for (int i=0;i<packets.Length;i++)
			{
				if (packets[i].Internal.IsReliable || packets[i].Internal.Seq == 0)
					continue;
				Lane lane = packets[i].Lane;
				if (lane.DoneHead - lane.DoneTail != lane.Done.Length)
				{
					uint head = lane.DoneHead % (uint)lane.Done.Length;
					lane.Done[head].Data = packets[i].Packet;
					lane.Done[head].Reliable = false;
					lane.Done[head].SeqId = packets[i].Internal.Seq;
					lane.Done[head].ArrivalTime = packets[i].ArrivalTime;
					lane.Done[head].CompletionTime = packets[i].ArrivalTime;
					lane.Done[head].Lane = lane;
					lane.DoneHead = lane.DoneHead + 1;
				}
			}

			// Handle reliable. These go straight
			for (int i=0;i<packets.Length;i++)
			{
				if (!packets[i].Internal.IsReliable || packets[i].Internal.Seq == 0)
					continue;

				Lane lane = packets[i].Lane;

				if (packets[i].Internal.Seq <= lane.ReliableSeq)
				{
					lane.Stats.RecvDupes++;
					continue;
				}

				// If packet arrives a second time, ignore.
				uint numProgress = (uint)lane.Progress.Length;
				bool hadit = false;
				for (uint p = lane.ProgressTail;p != lane.ProgressHead;p++)
				{
					uint idx = p % numProgress;
					if (lane.Progress[idx].SeqId == packets[i].Internal.Seq)
					{
						hadit = true;
						break;
					}
				}

				if (hadit)
				{
					// Already got the packet for this sequence id.
					lane.Stats.RecvDupes++;
					continue;
				}

				if (lane.ProgressHead - lane.ProgressTail != lane.Progress.Length)
				{
					uint head = lane.ProgressHead % (uint)lane.Progress.Length;
					lane.Progress[head].Data = packets[i].Packet;
					lane.Progress[head].IsFinalPiece = packets[i].Internal.IsFinalPiece;
					lane.Progress[head].ArrivalTime = packets[i].ArrivalTime;
					lane.Progress[head].SeqId = packets[i].Internal.Seq;
					lane.ProgressHead = lane.ProgressHead + 1;
				}
			}
		}

		// Return if there is more to come.
		public static bool ProcessLanesSend(LaneSetup setup, Lane[] lanes, DateTime now, Send[] output, out uint numOut)
		{
			numOut = 0;
			for (int i=0;i<lanes.Length;i++)
			{
				Lane lane = lanes[i];
				int holes = 0;
				for (int j=0;j<lane.OutCount;j++)
				{
					if (lane.Out[j].Source == null)
					{
						holes++;
						continue;
					}
					if (lane.Out[j].SendTime <= now)
					{
						lane.Stats.SendCount++;
						lane.Out[j].SendCount++;
						if (lane.Out[j].SendCount > 1)
						{
							lane.Stats.SendResends++;
						}

						uint resendMsAdd = lane.Out[j].SendCount * lane.Out[j].SendCount * lane.ResendMs;
						if (resendMsAdd > 5000)
						{
							resendMsAdd = 5000;
						}

						lane.Out[j].SendTime = lane.Out[j].SendTime.AddMilliseconds(resendMsAdd);

						Bitstream.Buffer tmp = setup.Factory.GetBuffer(setup.MaxPacketSize);
						tmp.bytepos = setup.ReservedHeaderBytes;

						Bitstream.PutBits(tmp, 1, lane.Out[j].Reliable ? 1u : 0u);
						Bitstream.PutCompressedUint(tmp, lane.Out[j].SeqId);

						if (lane.Out[j].Reliable)
						{
							Bitstream.PutBits(tmp, 1, lane.Out[j].IsFinalPiece ? 1u : 0u);

							// flush out a max number of acks or all.
							int wrote = 0;
							for (int k=0;k<lane.AckCount && k < 4;k++)
							{
								Bitstream.PutCompressedUint(tmp, lane.Acks[k]);
								wrote++;
							}
							Bitstream.PutCompressedUint(tmp, 0);
							for (int k=0;k<(int)lane.AckCount - wrote;k++)
							{
								lane.Acks[k] = lane.Acks[k + wrote];
							}
							lane.AckCount = (ushort)(lane.AckCount - wrote);
						}

						Bitstream.SyncByte(tmp);

						uint begin = lane.Out[j].Begin;
						uint end = lane.Out[j].End;
						uint ofs = tmp.bytepos;
						byte[] src = lane.Out[j].Source.buf;
						byte[] dst = tmp.buf;
						for (uint k=begin;k!=end;k++)
						{
							dst[ofs++] = src[k];
						}
						tmp.bytepos = 0;
						tmp.bitpos = 0;
						tmp.bytesize = ofs;
						tmp.bitsize = 0;
						output[numOut].Data = tmp;
						output[numOut].Lane = lane;
						lane.Stats.SendBytes += ofs;
						if (++numOut == output.Length)
						{
							return true;
						}
					}
				}

				if (holes > 0 && holes >= lane.OutCount / 4)
				{
					ushort write = 0;
					for (uint j=0;j<lane.OutCount;j++)
					{
						if (lane.Out[j].Source != null)
						{
							if (write != j)
							{
								lane.Out[write] = lane.Out[j];
							}
							++write;
						}
					}
					lane.OutCount = write;
				}
			}

			// Unsent acks
			for (int i=0;i<lanes.Length;i++)
			{
				Lane lane = lanes[i];
				if (lane.AckCount > 0)
				{
					Bitstream.Buffer tmp = setup.Factory.GetBuffer(setup.MaxPacketSize);
					tmp.bytepos = setup.ReservedHeaderBytes;
					Bitstream.PutBits(tmp, 1, 1); // reliable
					Bitstream.PutCompressedUint(tmp, 0); // ack only
					Bitstream.PutBits(tmp, 1, 1); // final

					for (int k=0;k<lane.AckCount;k++)
					{
						Bitstream.PutCompressedUint(tmp, lane.Acks[k]);
					}

					Bitstream.PutCompressedUint(tmp, 0);
					lane.AckCount = 0;
					Bitstream.SyncByte(tmp);
					tmp.Flip();
					output[numOut].Data = tmp;
					output[numOut].Lane = lane;

					lane.Stats.SendCount++;
					lane.Stats.SendAckOnly++;
					lane.Stats.SendBytes += tmp.bytesize;

					if (++numOut == output.Length)
					{
						return true;
					}
				}
			}

			return false;
		}

		// Will hold the buffers until they are sent
		public static void ScheduleSend(LaneSetup setup, DateTime now, ToSend[] tosend, uint count)
		{
			for (uint i=0;i<count;i++)
			{
				Lane lane = tosend[i].Lane;

				if (!tosend[i].Reliable)
				{
					uint target = (uint)lane.Out.Length;
					if (lane.OutCount == lane.Out.Length)
					{
						for (uint k=0;k<lane.Out.Length;k++)
						{
							if (lane.Out[k].Source == null)
							{
								target = k;
								break;
							}
						}
					}
					else
					{
						target = lane.OutCount++;
					}

					if (target == lane.Out.Length)
					{
						// Dropped
						continue;
					}

					if (tosend[i].CanKeepData)
					{
						lane.Out[target].Source = tosend[i].Data;
					}
					else
					{
						lane.Out[target].Source = setup.Factory.GetBuffer(tosend[i].Data.bytesize);
						Bitstream.Insert(lane.Out[target].Source, tosend[i].Data);
						Bitstream.SyncByte(lane.Out[target].Source);
						lane.Out[target].Source.Flip();
					}

					lane.Out[target].Source.userdata = 1; // refcount

					lane.Out[target].Begin = 0;
					lane.Out[target].End = tosend[i].Data.bytesize;
					lane.Out[target].IsFinalPiece = true;
					lane.Out[target].SendTime = now;
					lane.Out[target].InitialSendTime = now;
					lane.Out[target].SendCount = 0;
					lane.Out[target].SeqId = ++lane.OutgoingSeqUnreliable;
					lane.Out[target].Reliable = false;
				}
				else
				{
					uint segmentSize = setup.MaxPacketSize - 32 - setup.ReservedHeaderBytes;
					uint numSegments = (uint)(((tosend[i].Data.bytesize - tosend[i].Data.bytepos) + segmentSize - 1) / segmentSize);
					uint[] outSlots = new uint[256];
					if (numSegments == 0 || numSegments > outSlots.Length)
					{
						continue;
					}

					uint write = 0;
					for (uint k=0;k<lane.OutCount;k++)
					{
						if (lane.Out[k].Source == null)
						{
							outSlots[write++] = k;
							if (write == numSegments)
							{
								break;
							}
						}
					}

					uint left = numSegments - write;
					if ((lane.Out.Length - lane.OutCount) < left)
					{
						// no room.
						continue;
					}

					for (uint k=0;k<left;k++)
					{
						outSlots[write++] = lane.OutCount++;
					}

					Bitstream.Buffer source = tosend[i].Data;
					if (!tosend[i].CanKeepData)
					{
						source = setup.Factory.GetBuffer(tosend[i].Data.bytesize);
						Bitstream.Insert(source, tosend[i].Data);
						source.Flip();
					}

					source.userdata = 0;
					uint RangeBegin = 0;
					for (uint k=0;k<numSegments;k++)
					{
						uint bytesLeft = source.bytesize - RangeBegin;
						if (bytesLeft == 0)
						{
							Console.WriteLine("ERROR: bytesLeft=0!");
							continue;
						}
						uint toWrite = bytesLeft < segmentSize ? bytesLeft : segmentSize;
						uint slot = outSlots[k];
						source.userdata++;
						lane.Out[slot].Source = source;
						lane.Out[slot].Begin = RangeBegin;
						lane.Out[slot].End = RangeBegin + toWrite;
						lane.Out[slot].IsFinalPiece = k == (numSegments-1);
						lane.Out[slot].SendTime = now;
						lane.Out[slot].InitialSendTime = now;
						lane.Out[slot].SendCount = 0;
						lane.Out[slot].Reliable = true;
						lane.Out[slot].SeqId = ++lane.OutgoingSeqReliable;
						RangeBegin = RangeBegin + toWrite;
					}
				}
			}
		}

		// Return if there is more to come.
		public static bool ProcessLanesIncoming(LaneSetup setup, Lane[] lanes, Done[] output, out uint numOut)
		{
			numOut = 0;

			// Check the progress if we can pick another one.
			for (int i=0;i<lanes.Length;i++)
			{
				Lane lane = lanes[i];

				while (lane.DoneTail != lane.DoneHead)
				{
					uint t = lane.DoneTail % (uint)lane.Done.Length;
					output[numOut] = lane.Done[t];
					lane.DoneTail = lane.DoneTail + 1;
					if (++numOut == output.Length)
					{
						return true;
					}
				}

				if (lane.ProgressHead == lane.ProgressTail)
				{
					continue;
				}

				uint numProgress = (uint)lane.Progress.Length;

				// Clear out any seqId=0 (these are empty slots).
				while (lane.ProgressHead != lane.ProgressTail && lane.Progress[lane.ProgressTail % numProgress].SeqId == 0)
				{
					lane.ProgressTail++;
				}

				// And from the head too...
				while (lane.ProgressHead != lane.ProgressTail && lane.Progress[(numProgress + lane.ProgressHead-1) % numProgress].SeqId == 0)
				{
					lane.ProgressHead--;
				}

				if (lane.ProgressHead == lane.ProgressTail)
				{
					continue;
				}

				// Sort them by seq. There shouldn't be so many here. Maybe do something more clever if the count grows big.
				uint count = lane.ProgressHead - lane.ProgressTail;
				while (count > 1)
				{
					bool swapped = false;
					for (uint j = 0;j < (count-1); j++)
					{
						uint idx0 = (lane.ProgressTail + j) % numProgress;
						uint idx1 = (lane.ProgressTail + j + 1) % numProgress;
						if (lane.Progress[idx0].SeqId > lane.Progress[idx1].SeqId)
						{
							InProgress tmp = lane.Progress[idx0];
							lane.Progress[idx0] = lane.Progress[idx1];
							lane.Progress[idx1] = tmp;
							swapped = true;
						}
					}
					if (!swapped)
					{
						break;
					}
				}

				// Guaranteed to have them sorted in order now.
				uint head = lane.ProgressTail;
				uint next = lanes[i].ReliableSeq + 1;

				uint[] aggregateIdx = new uint[128];
				uint aggregateCount = 0;

				uint tail = lane.ProgressTail;
				while (tail != lane.ProgressHead)
				{
					uint idx = tail % numProgress;
					if (lane.Progress[idx].SeqId != next)
					{
						break;
					}
					if (lane.Progress[idx].IsFinalPiece)
					{
						if (aggregateCount == 0)
						{
							output[numOut].ArrivalTime = lane.Progress[idx].ArrivalTime;
							output[numOut].CompletionTime = lane.Progress[idx].ArrivalTime;
							output[numOut].Reliable = true;
							output[numOut].SeqId = lane.Progress[idx].SeqId;
							output[numOut].Data = lane.Progress[idx].Data;
							output[numOut].Lane = lane;
							lane.Progress[idx].SeqId = 0;
							lane.ReliableSeq = next;
							tail = tail + 1;
							next = next + 1;
							if (++numOut == output.Length)
							{
								return true;
							}
						}
						else
						{
							// There is always one room because check belowe makes sure of that.
							aggregateIdx[aggregateCount++] = idx;
							uint bits = 0;
							for (int k=0;k<aggregateCount;k++)
							{
								uint ki = aggregateIdx[k];
								bits = bits + lane.Progress[ki].Data.BitsLeft();
							}
							Bitstream.Buffer total = setup.Factory.GetBuffer(bits / 8 + 16);
							for (int k=0;k<aggregateCount;k++)
							{
								uint ki = aggregateIdx[k];
								Bitstream.Insert(total, lane.Progress[ki].Data);
								lane.Progress[ki].SeqId = 0;
							}
							total.Flip();
							output[numOut].ArrivalTime = lane.Progress[aggregateIdx[0]].ArrivalTime;
							output[numOut].CompletionTime = lane.Progress[idx].ArrivalTime;
							output[numOut].Reliable = true;
							output[numOut].SeqId = lane.Progress[idx].SeqId;
							output[numOut].Data = total;
							output[numOut].Lane = lane;
							lane.ReliableSeq = next;
							tail = tail + 1;
							next = next + 1;
							aggregateCount = 0;
							if (++numOut == output.Length)
							{
								return true;
							}
						}
					}
					else
					{
						aggregateIdx[aggregateCount] = idx;
						next = next + 1;
						tail = tail + 1;
						if (++aggregateCount >= (aggregateIdx.Length-1))
						{
							// THis should never happen.
							Console.WriteLine("ERROR: Progress buffer reset because aggregateIdx overflowed.");
							lane.ProgressHead = 0;
							lane.ProgressTail = 0;
							break;
						}
					}
				}
			}
			return false;
		}
	}
}