using System;

namespace Netki
{
	public static class PktLane
	{
		public struct PendingOut
		{
			public Bitstream.Buffer Segment;
			public uint SeqId;
			public bool IsFinalPiece;
			public DateTime SendTime;
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
			public bool SentAck;
		}

		public struct Done
		{
			public uint SeqId;
			public int LaneId;
			public bool Reliable;
			public Bitstream.Buffer Data;
			public DateTime ArrivalTime;
			public DateTime CompletionTime;
		}

		public class Lane
		{
			// id is user data.
			public Lane(int id, int slots)
			{
				Out = new PendingOut[slots];
				Progress = new InProgress[slots];
				Done = new Done[slots];
				Id = id;
			}
			public PendingOut[] Out;
			public InProgress[] Progress;
			public Done[] Done;
			public ushort OutCount;
			public uint ProgressHead;
			public uint ProgressTail;
			public uint DoneHead;
			public uint DoneTail;
			public uint ReliableSeq;
			public int Id;
		}

		public struct IncomingInternal
		{
			public uint Seq;
			public bool IsReliable;
			public bool IsFinalPiece;
		}

		public struct Incoming
		{
			public uint Lane;
			public Bitstream.Buffer Packet;
			public DateTime ArrivalTime;
			public IncomingInternal Internal;
		}

		public struct LaneSetup
		{
			public BufferFactory Factory;
		}

		// This will start reading from the buffers and modify the incoming array.
		public static void HandleIncomingPackets(LaneSetup setup, Lane[] lanes, Incoming[] packets, uint packetsCount)
		{
			// First pass extract.
			for (int i=0;i<packets.Length;i++)
			{
				Bitstream.Buffer tmp = packets[i].Packet;
				bool reliable = Bitstream.ReadBits(tmp, 1) == 1;
				uint seq = Bitstream.ReadCompressedUint(tmp);
				packets[i].Internal.Seq = seq;
				packets[i].Internal.IsReliable = reliable;
				packets[i].Internal.IsFinalPiece = reliable ? Bitstream.ReadBits(tmp, 1) == 1 : true;
			}

			// Unreliable packets, these are placed in the out queue after reading out the sequence and mode.
			for (int i=0;i<packets.Length;i++)
			{
				if (packets[i].Internal.IsReliable || packets[i].Internal.Seq == 0)
					continue;
				Lane lane = lanes[packets[i].Lane];
				if (lane.DoneHead - lane.DoneTail != lane.Done.Length)
				{
					uint head = lane.DoneHead % (uint)lane.Done.Length;
					lane.Done[head].Data = packets[i].Packet;
					lane.Done[head].Reliable = false;
					lane.Done[head].SeqId = packets[i].Internal.Seq;
					lane.Done[head].ArrivalTime = packets[i].ArrivalTime;
					lane.Done[head].CompletionTime = packets[i].ArrivalTime;
					lane.Done[head].LaneId = lane.Id;
					lane.DoneHead = lane.DoneHead + 1;
				}
			}

			// Handle reliable. These go straight
			for (int i=0;i<packets.Length;i++)
			{
				if (!packets[i].Internal.IsReliable || packets[i].Internal.Seq == 0)
					continue;
				
				Lane lane = lanes[packets[i].Lane];

				// If packet arrives a second time, send another ack back.
				uint numProgress = (uint)lane.Progress.Length;
				bool hadit = false;
				for (uint p = lane.ProgressTail;p != lane.ProgressHead;p++)
				{
					uint idx = p % numProgress;
					if (lane.Progress[idx].SeqId == packets[i].Internal.Seq)
					{
						lane.Progress[idx].SentAck = false;
						hadit = true;
					}
				}

				if (hadit)
				{
					// Already got the packet for this sequence id.
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
				while (lane.ProgressHead != lane.ProgressTail && lane.Progress[lane.ProgressTail].SeqId == 0)
				{
					lane.ProgressTail++;
				}

				// And from the head too...
				while (lane.ProgressHead != lane.ProgressTail && lane.Progress[lane.ProgressHead-1].SeqId == 0)
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
							output[numOut].LaneId = lane.Id;
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
							output[numOut].LaneId = lane.Id;

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